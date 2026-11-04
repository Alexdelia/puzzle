import struct

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

NODES = 64
NODE_FEATURES = 28
GLOBAL_FEATURES = 8
TRIANGLE_FEATURES = 12
MAX_LIVE = 96
HOLD = 0
SPAWN_BASE = 1
ATTACK_BASE = 4
TRIANGLE_ACTIONS = ATTACK_BASE + NODES
NET_VERSION = 1
PACK_BASE = 0x4E00

RECORD = np.dtype(
	[
		("turn", "<i4"),
		("count", "<i4"),
		("live", "<i4"),
		("nodes", "<f4", (NODES, NODE_FEATURES)),
		("adjacency", "<u8", (NODES,)),
		("units", "<i4", (NODES,)),
		("globals", "<f4", (GLOBAL_FEATURES,)),
		("corners", "<i4", (MAX_LIVE, 3)),
		("triangles", "<f4", (MAX_LIVE, TRIANGLE_FEATURES)),
		("attack", "<u8", (MAX_LIVE,)),
		("action", "<i4", (MAX_LIVE,)),
		("flows", "<i2", (NODES, NODES)),
	]
)


def unpack_bits(words):
	shifts = torch.arange(NODES, dtype=torch.int64, device=words.device)
	return ((words[..., None] >> shifts) & 1).bool()


def tensors(records, live_limit):
	def field(name, dtype, live=False):
		values = records[name][:, :live_limit] if live else records[name]
		return torch.from_numpy(
			np.ascontiguousarray(values).astype(dtype, copy=False).view(dtype)
		)

	return {
		"count": field("count", np.int64),
		"nodes": field("nodes", np.float32),
		"adjacency": torch.from_numpy(
			np.ascontiguousarray(records["adjacency"]).view(np.int64)
		),
		"globals": field("globals", np.float32),
		"corners": field("corners", np.int16, live=True),
		"triangles": field("triangles", np.float32, live=True),
		"attack": torch.from_numpy(
			np.ascontiguousarray(records["attack"][:, :live_limit]).view(np.int64)
		),
		"action": field("action", np.int16, live=True),
		"flows": field("flows", np.int16),
	}


def expand(compact):
	corners = compact["corners"].long()
	triangle_mask = corners[..., 0] >= 0
	return {
		"node_mask": torch.arange(NODES, device=corners.device)[None, :]
		< compact["count"][:, None],
		"nodes": compact["nodes"],
		"adjacency": unpack_bits(compact["adjacency"]).float(),
		"globals": compact["globals"],
		"corners": corners,
		"incidence": F.one_hot(corners.clamp(min=0), NODES).sum(2).float()
		* triangle_mask[..., None],
		"triangle_mask": triangle_mask,
		"triangles": compact["triangles"],
		"attack": unpack_bits(compact["attack"]),
		**({"action": compact["action"].long()} if "action" in compact else {}),
		**({"flows": compact["flows"].float()} if "flows" in compact else {}),
	}


class Round(nn.Module):
	def __init__(self, channels):
		super().__init__()
		self.own = nn.Linear(channels, channels)
		self.neighbors = nn.Linear(channels, channels)
		self.triangle = nn.Linear(channels + TRIANGLE_FEATURES, channels)
		self.gather = nn.Linear(channels, channels)


class Nova(nn.Module):
	def __init__(self, channels=32, rounds=3, attention=16):
		super().__init__()
		self.channels = channels
		self.embed = nn.Linear(NODE_FEATURES + GLOBAL_FEATURES, channels)
		self.rounds = nn.ModuleList(Round(channels) for _ in range(rounds))
		self.query = nn.Linear(channels, attention)
		self.key = nn.Linear(channels, attention)
		self.destination = nn.Linear(channels, 1)
		self.stay = nn.Linear(channels, 1)
		self.triangle = nn.Linear(channels + TRIANGLE_FEATURES, channels)
		self.hold = nn.Linear(channels, 1)
		self.spawn = nn.Linear(2 * channels, 1)
		self.attack = nn.Linear(2 * channels, 1)
		self.value_hidden = nn.Linear(2 * channels + GLOBAL_FEATURES, 64)
		self.value_out = nn.Linear(64, 1)

	def layers(self):
		yield self.embed
		for round_ in self.rounds:
			yield round_.own
			yield round_.neighbors
			yield round_.triangle
			yield round_.gather
		for layer in (
			self.query,
			self.key,
			self.destination,
			self.stay,
			self.triangle,
			self.hold,
			self.spawn,
			self.attack,
		):
			yield layer

	def hidden(self, batch):
		nodes = batch["nodes"]
		size = nodes.shape[0]
		globals_ = batch["globals"][:, None, :].expand(size, NODES, GLOBAL_FEATURES)
		h = F.relu(self.embed(torch.cat([nodes, globals_], 2)))
		adjacency = batch["adjacency"]
		degree = adjacency.sum(2, keepdim=True).clamp(min=1.0)
		incidence = batch["incidence"]
		triangle_mask = batch["triangle_mask"][..., None].float()
		per_node = incidence.transpose(1, 2) @ triangle_mask
		for round_ in self.rounds:
			mean = adjacency @ h / degree
			corner_mean = incidence @ h / 3.0
			message = (
				F.relu(round_.triangle(torch.cat([corner_mean, batch["triangles"]], 2)))
				* triangle_mask
			)
			gathered = incidence.transpose(1, 2) @ message / per_node.clamp(min=1.0)
			h = (
				F.relu(round_.own(h) + round_.neighbors(mean) + round_.gather(gathered))
				+ h
			)
		return h

	def forward(self, batch):
		move, triangle, _ = self.evaluate(batch, with_value=False)
		return move, triangle

	def value(self, batch, h):
		mask = batch["node_mask"][..., None].float()
		mean = (h * mask).sum(1) / mask.sum(1).clamp(min=1.0)
		peak = h.masked_fill(mask == 0, -1e9).amax(1)
		pooled = torch.cat([mean, peak, batch["globals"]], 1)
		return self.value_out(F.relu(self.value_hidden(pooled))).squeeze(1)

	def evaluate(self, batch, with_value=True):
		h = self.hidden(batch)
		query = self.query(h)
		key = self.key(h)
		pair = query @ key.transpose(1, 2)
		eye = torch.eye(NODES, dtype=torch.bool, device=h.device)
		move = pair + self.destination(h).transpose(1, 2)
		move = torch.where(eye, pair + self.stay(h), move)
		allowed = (batch["adjacency"] > 0) | eye
		move = move.masked_fill(~allowed, -1e9)
		incidence = batch["incidence"]
		corner_mean = incidence @ h / 3.0
		z = F.relu(self.triangle(torch.cat([corner_mean, batch["triangles"]], 2)))
		hold = self.hold(z)
		corners = batch["corners"].clamp(min=0)
		corner_h = torch.gather(
			h[:, None, :, :].expand(-1, corners.shape[1], -1, -1),
			2,
			corners[..., None].expand(-1, -1, -1, self.channels),
		)
		spawn = self.spawn(
			torch.cat([z[:, :, None, :].expand(-1, -1, 3, -1), corner_h], 3)
		).squeeze(3)
		weight_z, weight_h = (
			self.attack.weight[:, : self.channels],
			self.attack.weight[:, self.channels :],
		)
		attack = (
			(z @ weight_z.t()) + (h @ weight_h.t()).transpose(1, 2) + self.attack.bias
		)
		attack = attack.masked_fill(~batch["attack"], -1e9)
		triangle = torch.cat([hold, spawn, attack], 2)
		return move, triangle, self.value(batch, h) if with_value else None


def losses(model, batch):
	move, triangle = model(batch)
	flows = batch["flows"]
	log_move = F.log_softmax(move, 2)
	units = flows.sum()
	move_loss = -(flows * log_move.masked_fill(flows == 0, 0.0)).sum() / units.clamp(
		min=1.0
	)
	action = batch["action"]
	labelled = action >= 0
	triangle_loss = (
		F.cross_entropy(triangle[labelled], action[labelled])
		if labelled.any()
		else move_loss * 0
	)
	return move_loss, triangle_loss, move, triangle


def quantise(matrix):
	peak = np.abs(matrix).max(axis=1)
	step = np.where(peak > 0, peak / 127.0, 1.0).astype(np.float32)
	return np.clip(np.rint(matrix / step[:, None]), -127, 127).astype(np.int8), step


def export(model):
	payload = bytearray(
		struct.pack("<BBB", NET_VERSION, model.channels, len(model.rounds))
	)
	for layer in [*model.layers(), model.value_hidden, model.value_out]:
		weight = layer.weight.detach().cpu().numpy().astype(np.float32)
		bias = layer.bias.detach().cpu().numpy().astype(np.float32)
		values, step = quantise(weight)
		payload += struct.pack("<HH", weight.shape[0], weight.shape[1])
		payload += values.tobytes()
		payload += step.astype("<f4").tobytes()
		payload += bias.astype("<f4").tobytes()
	return bytes(payload)


def dequantise(model):
	with torch.no_grad():
		for layer in model.layers():
			weight = layer.weight.detach().cpu().numpy().astype(np.float32)
			values, step = quantise(weight)
			layer.weight.copy_(
				torch.from_numpy(values.astype(np.float32) * step[:, None])
			)


def pack(payload):
	out = []
	bits = 0
	have = 0
	for byte in payload:
		bits = (bits << 8) | byte
		have += 8
		while have >= 14:
			have -= 14
			out.append(chr(PACK_BASE + ((bits >> have) & 0x3FFF)))
	if have:
		out.append(chr(PACK_BASE + ((bits << (14 - have)) & 0x3FFF)))
	return "".join(out)
