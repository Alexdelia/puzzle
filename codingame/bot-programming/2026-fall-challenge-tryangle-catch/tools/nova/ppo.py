import argparse
import collections
import json
import pathlib
import sys
import time

import numpy as np
import torch
import torch.nn.functional as F

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import novaenv
from model import (
	GLOBAL_FEATURES,
	MAX_LIVE,
	NODE_FEATURES,
	NODES,
	TRIANGLE_ACTIONS,
	TRIANGLE_FEATURES,
	Nova,
	expand,
	export,
)

USABLE_FEATURE = 5
MAX_SAMPLED_UNITS = 128


def buffers_for(slots):
	return {
		"nodes": np.zeros((slots, NODES, NODE_FEATURES), np.float32),
		"adjacency": np.zeros((slots, NODES), np.int64),
		"units": np.zeros((slots, NODES), np.int32),
		"globals": np.zeros((slots, GLOBAL_FEATURES), np.float32),
		"corners": np.zeros((slots, MAX_LIVE, 3), np.int32),
		"triangles": np.zeros((slots, MAX_LIVE, TRIANGLE_FEATURES), np.float32),
		"attack": np.zeros((slots, MAX_LIVE), np.int64),
		"count": np.zeros(slots, np.int32),
		"controlled": np.zeros(slots, np.uint8),
		"reward": np.zeros(slots, np.float32),
		"done": np.zeros(slots, np.uint8),
		"scores": np.zeros((slots, 2), np.int32),
		"moves": np.zeros((slots, NODES, NODES), np.int32),
		"actions": np.zeros((slots, MAX_LIVE), np.int32),
	}


OBSERVED = (
	"nodes",
	"adjacency",
	"units",
	"globals",
	"corners",
	"triangles",
	"attack",
	"count",
)


def observation(buffers, device):
	return {
		key: torch.from_numpy(buffers[key]).to(device, non_blocking=True)
		for key in OBSERVED
	}


def batch_of(compact):
	batch = expand({**compact, "count": compact["count"].long()})
	batch["units"] = compact["units"].long()
	batch["usable"] = (compact["triangles"][..., USABLE_FEATURE] > 0.5) & batch[
		"triangle_mask"
	]
	return batch


def move_log_probs(move, units):
	log_move = F.log_softmax(move, 2)
	active = units > 0
	return log_move, active


def sample(model, compact, generator):
	batch = batch_of(compact)
	with torch.no_grad():
		move, triangle, value = model.evaluate(batch)
		log_move, active = move_log_probs(move, batch["units"])
		size = move.shape[0]
		probs = log_move.exp().reshape(size * NODES, NODES)
		units = batch["units"].clamp(max=MAX_SAMPLED_UNITS).reshape(-1)
		draws = int(units.max().item()) if units.numel() else 0
		counts = torch.zeros(size * NODES, NODES, device=move.device)
		if draws > 0:
			picked = torch.multinomial(
				probs, draws, replacement=True, generator=generator
			)
			keep = (
				torch.arange(draws, device=move.device)[None, :] < units[:, None]
			).float()
			counts.scatter_add_(1, picked, keep)
		counts = counts.reshape(size, NODES, NODES)
		log_triangle = F.log_softmax(triangle, 2)
		chosen = torch.multinomial(
			log_triangle.exp().reshape(-1, TRIANGLE_ACTIONS), 1, generator=generator
		).reshape(size, -1)
		chosen = torch.where(batch["usable"], chosen, torch.full_like(chosen, -1))
		logp = policy_log_prob(
			log_move, active, counts, log_triangle, batch["usable"], chosen
		)
	return counts, chosen, logp, value


def policy_log_prob(log_move, active, counts, log_triangle, usable, chosen):
	moves = (counts * log_move.masked_fill(counts == 0, 0.0)).sum((1, 2))
	picked = log_triangle.gather(2, chosen.clamp(min=0)[..., None]).squeeze(2)
	triangles = (picked * usable.float()).sum(1)
	return moves + triangles


def entropy_of(log_move, active, log_triangle, usable):
	move_entropy = -(log_move.exp() * log_move.masked_fill(log_move < -1e8, 0.0)).sum(2)
	move_entropy = (move_entropy * active.float()).sum(1)
	triangle_entropy = -(
		log_triangle.exp() * log_triangle.masked_fill(log_triangle < -1e8, 0.0)
	).sum(2)
	triangle_entropy = (triangle_entropy * usable.float()).sum(1)
	return move_entropy + triangle_entropy


class ReturnScaler:
	def __init__(self, slots, gamma):
		self.gamma = gamma
		self.running = np.zeros(slots)
		self.count = 1e-4
		self.mean = 0.0
		self.var = 1.0

	def update(self, reward, done, controlled):
		self.running = self.running * self.gamma + reward
		batch = self.running[controlled]
		if len(batch):
			delta = batch.mean() - self.mean
			total = self.count + len(batch)
			self.mean += delta * len(batch) / total
			self.var = (
				self.var * self.count
				+ batch.var() * len(batch)
				+ delta**2 * self.count * len(batch) / total
			) / total
			self.count = total
		self.running[done] = 0.0

	def scale(self):
		return float(max(np.sqrt(self.var), 1e-3))


class Store:
	def __init__(self, steps, slots, device):
		shape = (steps, slots)
		self.data = {
			"nodes": torch.zeros(
				shape + (NODES, NODE_FEATURES), dtype=torch.float16, device=device
			),
			"adjacency": torch.zeros(
				shape + (NODES,), dtype=torch.int64, device=device
			),
			"units": torch.zeros(shape + (NODES,), dtype=torch.int16, device=device),
			"globals": torch.zeros(shape + (GLOBAL_FEATURES,), device=device),
			"corners": torch.zeros(
				shape + (MAX_LIVE, 3), dtype=torch.int16, device=device
			),
			"triangles": torch.zeros(
				shape + (MAX_LIVE, TRIANGLE_FEATURES),
				dtype=torch.float16,
				device=device,
			),
			"attack": torch.zeros(
				shape + (MAX_LIVE,), dtype=torch.int64, device=device
			),
			"count": torch.zeros(shape, dtype=torch.int16, device=device),
			"counts": torch.zeros(
				shape + (NODES, NODES), dtype=torch.int16, device=device
			),
			"chosen": torch.zeros(
				shape + (MAX_LIVE,), dtype=torch.int16, device=device
			),
			"controlled": torch.zeros(shape, dtype=torch.bool, device=device),
			"logp": torch.zeros(shape, device=device),
			"value": torch.zeros(shape, device=device),
			"reward": torch.zeros(shape, device=device),
			"done": torch.zeros(shape, device=device),
		}

	def put(self, t, compact, counts, chosen, controlled, logp, value):
		for key in OBSERVED:
			self.data[key][t] = compact[key].to(self.data[key].dtype)
		self.data["counts"][t] = counts.to(torch.int16)
		self.data["chosen"][t] = chosen.to(torch.int16)
		self.data["controlled"][t] = controlled
		self.data["logp"][t] = logp
		self.data["value"][t] = value


def compact_from(store, index):
	flat = {
		key: store.data[key].reshape(-1, *store.data[key].shape[2:])[index]
		for key in OBSERVED
	}
	flat["nodes"] = flat["nodes"].float()
	flat["triangles"] = flat["triangles"].float()
	flat["units"] = flat["units"].int()
	flat["count"] = flat["count"].int()
	return flat


def label(opponent):
	return (
		pathlib.Path(opponent.split()[-1]).stem
		if opponent.startswith(("bot:", "frozen:"))
		else opponent
	)


def save(model, optimiser, update, config, path):
	torch.save(
		{
			"model": model.state_dict(),
			"optimiser": optimiser.state_dict(),
			"update": update,
			"config": config,
		},
		path,
	)
	path.with_suffix(".bin").write_bytes(export(model))


def main():
	parser = argparse.ArgumentParser(
		description="PPO fine-tuning of a nova net in the exact referee"
	)
	parser.add_argument(
		"--init",
		required=True,
		help="checkpoint (.pt) to start from, e.g. a behaviour clone",
	)
	parser.add_argument("--out", required=True, help="run directory")
	parser.add_argument(
		"--opponents",
		default="self",
		help="comma list of self, frozen:<weights.bin>, bot:<command>; games cycle through it",
	)
	parser.add_argument("--games", type=int, default=128)
	parser.add_argument("--threads", type=int, default=16)
	parser.add_argument("--steps", type=int, default=32)
	parser.add_argument("--epochs", type=int, default=3)
	parser.add_argument("--minibatch", type=int, default=1024)
	parser.add_argument("--lr", type=float, default=1e-4)
	parser.add_argument("--clip", type=float, default=0.1)
	parser.add_argument("--entropy", type=float, default=0.003)
	parser.add_argument("--value-coef", type=float, default=0.5)
	parser.add_argument("--gamma", type=float, default=0.995)
	parser.add_argument("--lam", type=float, default=0.95)
	parser.add_argument("--target-kl", type=float, default=0.02)
	parser.add_argument(
		"--margin-scale",
		type=float,
		default=0.1,
		help="reward per point of score margin gained in a turn",
	)
	parser.add_argument("--win-bonus", type=float, default=3.0)
	parser.add_argument("--value-warmup", type=int, default=8)
	parser.add_argument("--updates", type=int, default=100000)
	parser.add_argument("--save-every", type=int, default=10)
	parser.add_argument("--seed", type=int, default=5000000)
	args = parser.parse_args()
	device = "cuda" if torch.cuda.is_available() else "cpu"
	out = pathlib.Path(args.out)
	out.mkdir(parents=True, exist_ok=True)
	state = torch.load(args.init, map_location=device, weights_only=False)
	config = state["config"]
	model = Nova(config["channels"], config["rounds"]).to(device)
	missing, unexpected = model.load_state_dict(state["model"], strict=False)
	assert not unexpected and all(key.startswith("value_") for key in missing), (
		missing,
		unexpected,
	)
	optimiser = torch.optim.Adam(model.parameters(), lr=args.lr, eps=1e-5)
	update0 = state.get("update", 0) if "optimiser" in state else 0
	if "optimiser" in state:
		optimiser.load_state_dict(state["optimiser"])
		for group in optimiser.param_groups:
			group["lr"] = args.lr
	opponents = args.opponents.split(",")
	env = novaenv.VecEnv(args.games, args.seed, opponents, args.threads)
	kinds = env.opponent_of()
	slots = env.slots
	buffers = buffers_for(slots)
	env.observe(buffers)
	store = Store(args.steps, slots, device)
	generator = torch.Generator(device=device)
	generator.manual_seed(args.seed)
	log = open(out / "log.jsonl", "a")
	scaler = ReturnScaler(slots, args.gamma)
	results = collections.defaultdict(lambda: collections.deque(maxlen=200))
	started = time.time()
	print(
		f"slots {slots} opponents {opponents} device {device} from {args.init}",
		flush=True,
	)
	for update in range(update0 + 1, update0 + args.updates + 1):
		warming = update - update0 <= args.value_warmup
		for name, parameter in model.named_parameters():
			parameter.requires_grad_(not warming or name.startswith("value_"))
		model.eval()
		rollout_started = time.time()
		for t in range(args.steps):
			compact = observation(buffers, device)
			counts, chosen, logp, value = sample(model, compact, generator)
			controlled = torch.from_numpy(buffers["controlled"].astype(bool)).to(device)
			store.put(t, compact, counts, chosen, controlled, logp, value)
			buffers["moves"][:] = counts.to(torch.int32).cpu().numpy()
			buffers["actions"][:] = chosen.to(torch.int32).cpu().numpy()
			env.step(buffers)
			reward = buffers["reward"] * args.margin_scale
			ended = buffers["done"].astype(bool)
			margin = buffers["scores"][:, 0] - buffers["scores"][:, 1]
			reward = reward + ended * args.win_bonus * np.sign(margin)
			scaler.update(reward, ended, controlled.cpu().numpy())
			store.data["reward"][t] = torch.from_numpy(reward.astype(np.float32)).to(
				device
			)
			store.data["done"][t] = torch.from_numpy(ended.astype(np.float32)).to(
				device
			)
			for slot in np.flatnonzero(ended & controlled.cpu().numpy()):
				kind = label(opponents[kinds[slot // 2]])
				results[kind].append(
					1.0 if margin[slot] > 0 else 0.5 if margin[slot] == 0 else 0.0
				)
				results[kind + " margin"].append(float(margin[slot]))
		rollout_seconds = time.time() - rollout_started
		with torch.no_grad():
			next_value = model.evaluate(batch_of(observation(buffers, device)))[2]
		rewards = store.data["reward"] / scaler.scale()
		values = store.data["value"]
		dones = store.data["done"]
		advantages = torch.zeros_like(rewards)
		last = torch.zeros(slots, device=device)
		for t in reversed(range(args.steps)):
			following = next_value if t == args.steps - 1 else values[t + 1]
			alive = 1.0 - dones[t]
			delta = rewards[t] + args.gamma * following * alive - values[t]
			last = delta + args.gamma * args.lam * alive * last
			advantages[t] = last
		returns = advantages + values
		controlled_index = torch.nonzero(store.data["controlled"].reshape(-1)).squeeze(
			1
		)
		flat_adv = advantages.reshape(-1)
		flat_ret = returns.reshape(-1)
		flat_logp = store.data["logp"].reshape(-1)
		flat_counts = store.data["counts"].reshape(-1, NODES, NODES)
		flat_chosen = store.data["chosen"].reshape(-1, MAX_LIVE)
		model.train()
		stats = collections.Counter()
		batches = 0
		stop = False
		for _ in range(args.epochs):
			if stop:
				break
			order = controlled_index[
				torch.randperm(len(controlled_index), device=device)
			]
			for start in range(0, len(order), args.minibatch):
				index = order[start : start + args.minibatch]
				batch = batch_of(compact_from(store, index))
				move, triangle, value = model.evaluate(batch)
				log_move, active = move_log_probs(move, batch["units"])
				log_triangle = F.log_softmax(triangle, 2)
				chosen = flat_chosen[index].long()
				logp = policy_log_prob(
					log_move,
					active,
					flat_counts[index].float(),
					log_triangle,
					batch["usable"],
					chosen,
				)
				entropy = entropy_of(
					log_move, active, log_triangle, batch["usable"]
				).mean()
				ratio = (logp - flat_logp[index]).exp()
				advantage = flat_adv[index]
				advantage = (advantage - advantage.mean()) / (advantage.std() + 1e-8)
				policy_loss = torch.max(
					-advantage * ratio,
					-advantage * ratio.clamp(1 - args.clip, 1 + args.clip),
				).mean()
				value_loss = 0.5 * (value - flat_ret[index]).pow(2).mean()
				loss = (
					value_loss
					if warming
					else policy_loss
					+ args.value_coef * value_loss
					- args.entropy * entropy
				)
				optimiser.zero_grad(set_to_none=True)
				loss.backward()
				torch.nn.utils.clip_grad_norm_(model.parameters(), 0.5)
				optimiser.step()
				with torch.no_grad():
					kl = (flat_logp[index] - logp).mean().item()
					stats["policy"] += policy_loss.item()
					stats["value"] += value_loss.item()
					stats["entropy"] += entropy.item()
					stats["kl"] += kl
					stats["clip"] += (
						((ratio - 1).abs() > args.clip).float().mean().item()
					)
				batches += 1
				if args.target_kl and not warming and abs(kl) > args.target_kl:
					stop = True
					break
		for key in stats:
			stats[key] /= max(batches, 1)
		elapsed = time.time() - started
		summary = {
			key: float(np.mean(values_)) for key, values_ in results.items() if values_
		}
		record = {
			"update": update,
			"elapsed_min": elapsed / 60,
			"rollout_s": rollout_seconds,
			"samples": int(len(controlled_index)),
			"mean_reward": float(store.data["reward"].mean().item()),
			"return_scale": scaler.scale(),
			"games_finished": int(env.finished),
			"warming": warming,
			**dict(stats),
			**summary,
		}
		log.write(json.dumps(record) + "\n")
		log.flush()
		shown = " ".join(f"{key} {value:.3f}" for key, value in summary.items())
		print(
			f"upd {update} roll {rollout_seconds:.1f}s samples {len(controlled_index)} games {env.finished} "
			f"scale {scaler.scale():.2f} ent {stats['entropy']:.2f} kl {stats['kl']:.4f} clip {stats['clip']:.3f} vloss {stats['value']:.3f} | {shown}",
			flush=True,
		)
		if update % args.save_every == 0:
			save(model, optimiser, update, config, out / "latest.pt")
			save(model, optimiser, update, config, out / f"upd{update:05d}.pt")


if __name__ == "__main__":
	main()
