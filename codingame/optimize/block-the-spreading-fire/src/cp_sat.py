import heapq
import os
import sys
from dataclasses import dataclass
from pathlib import Path

from ortools.sat.python import cp_model


@dataclass
class Params:
	cut: int
	fire: int
	value: int


@dataclass
class Map:
	width: int
	tree: Params
	house: Params
	start: int
	grid: str

	def params(self, i: int) -> Params:
		return self.house if self.grid[i] == "X" else self.tree

	def neighbours(self, i: int) -> list[int]:
		around = (i - self.width, i - 1, i + 1, i + self.width)
		return [n for n in around if self.grid[n] != "#"]

	def total_value(self) -> int:
		cells = (i for i, cell in enumerate(self.grid) if cell != "#")
		return sum(self.params(i).value for i in cells)


def parse(path: str) -> Map:
	lines = Path(path).read_text().splitlines()
	tree = Params(*map(int, lines[0].split()))
	house = Params(*map(int, lines[1].split()))
	width, height = map(int, lines[2].split())
	start_x, start_y = map(int, lines[3].split())
	grid = "".join(row[:width] for row in lines[4 : 4 + height])
	return Map(width, tree, house, start_y * width + start_x, grid)


def earliest_ignition(m: Map) -> dict[int, int]:
	ignition = {m.start: 0}
	queue = [(0, m.start)]
	while queue:
		time, i = heapq.heappop(queue)
		if time > ignition[i]:
			continue
		for n in m.neighbours(i):
			arrival = time + m.params(i).fire
			if arrival < ignition.get(n, arrival + 1):
				ignition[n] = arrival
				heapq.heappush(queue, (arrival, n))
	return ignition


def read_cut_turns(m: Map, path: str) -> dict[int, int]:
	turns = {}
	turn = 1
	for line in Path(path).read_text().splitlines():
		x, y = map(int, line.split())
		i = y * m.width + x
		turns[i] = turn
		turn += m.params(i).cut
	return turns


class TimedModel:
	def __init__(self, m: Map, horizon: int) -> None:
		self.m = m
		self.horizon = horizon
		self.ignition = earliest_ignition(m)
		self.cells = sorted(self.ignition)
		self.model = cp_model.CpModel()
		zero = self.model.new_constant(0)
		self.fire = {}
		self.cut = {}
		for i in self.cells:
			for t in range(horizon + 1):
				reachable = t >= self.ignition[i]
				self.fire[i, t] = (
					self.model.new_bool_var(f"f{i}_{t}") if reachable else zero
				)
				self.cut[i, t] = (
					self.model.new_bool_var(f"c{i}_{t}") if t >= 1 else zero
				)
		self.burnt = {i: self.model.new_bool_var(f"b{i}") for i in self.cells}
		self.late_cut = {i: self.model.new_bool_var(f"l{i}") for i in self.cells}
		self.add_spread()
		self.add_aftermath()
		self.add_cooldown()
		self.model.minimize(sum(self.lost(i) for i in self.cells))

	def add_spread(self) -> None:
		fire, cut = self.fire, self.cut
		self.model.add(fire[self.m.start, 0] == 1)
		for i in self.cells:
			for t in range(1, self.horizon + 1):
				self.model.add(cut[i, t] >= cut[i, t - 1])
				self.model.add(fire[i, t] >= fire[i, t - 1])
				self.model.add(cut[i, t] - cut[i, t - 1] + fire[i, t - 1] <= 1)
				self.model.add(cut[i, t] + fire[i, t] <= 1)
				for n in self.m.neighbours(i):
					before = t - self.m.params(n).fire
					if before >= self.ignition[n]:
						self.model.add(fire[i, t] >= fire[n, before] - cut[i, t])

	def add_aftermath(self) -> None:
		end = self.horizon
		for i in self.cells:
			burnt, late_cut, cut = self.burnt[i], self.late_cut[i], self.cut[i, end]
			self.model.add(burnt >= self.fire[i, end])
			self.model.add(late_cut + self.fire[i, end] <= 1)
			self.model.add(burnt + cut + late_cut <= 1)
			for n in self.m.neighbours(i):
				self.model.add(burnt >= self.burnt[n] - cut - late_cut)

	def add_cooldown(self) -> None:
		for t in range(1, self.horizon + 1):
			cutting = (
				self.cut[i, t] - self.cut[i, max(t - self.m.params(i).cut, 0)]
				for i in self.cells
			)
			self.model.add(sum(cutting) <= 1)

	def lost(self, i: int) -> cp_model.LinearExpr:
		gone = self.burnt[i] + self.cut[i, self.horizon] + self.late_cut[i]
		return self.m.params(i).value * gone

	def add_hint(self, hint_turns: dict[int, int]) -> None:
		for i in self.cells:
			for t in range(1, self.horizon + 1):
				self.model.add_hint(
					self.cut[i, t], i in hint_turns and hint_turns[i] <= t
				)

	def solve(self, seconds: float) -> tuple[str, cp_model.CpSolver]:
		solver = cp_model.CpSolver()
		solver.parameters.max_time_in_seconds = seconds
		solver.parameters.num_workers = os.cpu_count()
		status = solver.solve(self.model)
		return solver.status_name(status), solver

	def cut_order(self, solver: cp_model.CpSolver) -> list[int]:
		turns = range(1, self.horizon + 1)
		cut_turns = {
			i: next(t for t in turns if solver.value(self.cut[i, t]))
			for i in self.cells
			if solver.value(self.cut[i, self.horizon])
		}
		return sorted(cut_turns, key=cut_turns.get)

	def late_cuts(self, solver: cp_model.CpSolver) -> int:
		return sum(solver.value(self.late_cut[i]) for i in self.cells)


def main() -> None:
	if len(sys.argv) < 5:
		sys.exit("usage: cp_sat.py <map> <horizon> <seconds> <out> [hint]")
	map_path, horizon, seconds, out_path = sys.argv[1:5]
	m = parse(map_path)
	timed = TimedModel(m, int(horizon))
	if len(sys.argv) > 5:
		timed.add_hint(read_cut_turns(m, sys.argv[5]))
	status, solver = timed.solve(float(seconds))
	total = m.total_value()
	best = total - round(solver.objective_value)
	upper_bound = total - round(solver.best_objective_bound)
	print(
		f"{map_path} {status} best {best} upper bound {upper_bound}"
		f" late cuts {timed.late_cuts(solver)}"
	)
	lines = (f"{i % m.width} {i // m.width}\n" for i in timed.cut_order(solver))
	Path(out_path).write_text("".join(lines))


if __name__ == "__main__":
	main()
