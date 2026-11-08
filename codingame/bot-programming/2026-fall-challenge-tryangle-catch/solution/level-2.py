import time
from collections import deque

TIME_BUDGET = 0.035
MAX_ARMY = 20
PROGRESS_WEIGHT = 0.2
GOAL_WEIGHT = 1.0
IDLE_WEIGHT = 0.3
OPPONENT_TRIANGLE_BONUS = 0.5
UNIT_VALUE = 1.0
TRIANGLE_VALUE = 0.0
ATTACK_MARGIN = 0.5
MAX_ATTACK_CANDIDATES = 4
MIN_KILLS = 3
MAX_ATTACKS = 3


def read_ints():
	return list(map(int, input().split()))


def bfs_all(adjacency):
	count = len(adjacency)
	distances = []
	for source in range(count):
		dist = [99] * count
		dist[source] = 0
		queue = deque([source])
		while queue:
			node = queue.popleft()
			for next_node in adjacency[node]:
				if dist[next_node] == 99:
					dist[next_node] = dist[node] + 1
					queue.append(next_node)
		distances.append(dist)
	return distances


def node_owner(node, mine, theirs, adjacency):
	neighbors = adjacency[node]
	if neighbors:
		if all(mine[n] > theirs[n] for n in neighbors):
			return 0
		if all(theirs[n] > mine[n] for n in neighbors):
			return 1
	if mine[node] > theirs[node]:
		return 0
	if theirs[node] > mine[node]:
		return 1
	return -1


class Turn:
	def __init__(self, house_count):
		self.my_score, self.opp_score = read_ints()
		self.mine = [0] * house_count
		self.theirs = [0] * house_count
		for _ in range(house_count):
			house, my_units, opp_units = read_ints()
			self.mine[house] = my_units
			self.theirs[house] = opp_units
		self.adjacency = [[] for _ in range(house_count)]
		for _ in range(int(input())):
			a, b = read_ints()
			self.adjacency[a].append(b)
			self.adjacency[b].append(a)
		self.triangles = []
		for _ in range(int(input())):
			a, b, c, owner, me_can, opp_can = read_ints()
			self.triangles.append(((a, b, c), owner, me_can == 1, opp_can == 1))
		for _ in range(int(input())):
			input()

	def after_attack(self, triangle_index, target):
		after = object.__new__(Turn)
		after.my_score, after.opp_score = self.my_score, self.opp_score
		after.adjacency = self.adjacency
		after.mine = list(self.mine)
		after.theirs = list(self.theirs)
		corners, _, _, opp_can = self.triangles[triangle_index]
		for corner in corners:
			after.mine[corner] -= 1
		after.theirs[target] = 0
		after.triangles = list(self.triangles)
		after.triangles[triangle_index] = (corners, -1, False, opp_can)
		return after


class Position:
	def __init__(self, turn, mine):
		self.adjacency = turn.adjacency
		self.theirs = turn.theirs
		self.triangles = turn.triangles
		self.node_triangles = [[] for _ in mine]
		for index, (corners, _, _, _) in enumerate(turn.triangles):
			for corner in corners:
				self.node_triangles[corner].append(index)
		self.mine = list(mine)
		nodes = range(len(mine))
		self.ahead = [self.mine[n] > self.theirs[n] for n in nodes]
		self.behind = [self.theirs[n] > self.mine[n] for n in nodes]
		self.ahead_around = [
			sum(self.ahead[m] for m in self.adjacency[n]) for n in nodes
		]
		self.behind_around = [
			sum(self.behind[m] for m in self.adjacency[n]) for n in nodes
		]
		self.owners = [self.owner_of(n) for n in nodes]
		self.penalties = [self.penalty_of(n) for n in nodes]
		self.triangle_values = [
			self.triangle_value(t) for t in range(len(self.triangles))
		]
		self.value = sum(self.penalties) + sum(self.triangle_values)

	def owner_of(self, node):
		degree = len(self.adjacency[node])
		if degree:
			if self.ahead_around[node] == degree:
				return 0
			if self.behind_around[node] == degree:
				return 1
		if self.ahead[node]:
			return 0
		if self.behind[node]:
			return 1
		return -1

	def penalty_of(self, node):
		count = self.mine[node]
		degree = len(self.adjacency[node])
		if count and degree and self.behind_around[node] == degree:
			return -8 * count
		return 0

	def triangle_value(self, index):
		corners, owner, me_can, opp_can = self.triangles[index]
		owners = self.owners
		a, b, c = corners
		if owner == 0:
			if opp_can and owners[a] == 1 and owners[b] == 1 and owners[c] == 1:
				return -10
			return 0
		if not me_can:
			return 0
		owned_corners = (owners[a] == 0) + (owners[b] == 0) + (owners[c] == 0)
		if owned_corners == 3:
			return 12 if owner == 1 else 10
		return PROGRESS_WEIGHT * owned_corners * owned_corners

	def shift(self, node, amount, dirty):
		self.mine[node] += amount
		dirty.add(node)
		ahead = self.mine[node] > self.theirs[node]
		behind = self.theirs[node] > self.mine[node]
		if ahead != self.ahead[node]:
			self.ahead[node] = ahead
			step = 1 if ahead else -1
			for neighbor in self.adjacency[node]:
				self.ahead_around[neighbor] += step
				dirty.add(neighbor)
		if behind != self.behind[node]:
			self.behind[node] = behind
			step = 1 if behind else -1
			for neighbor in self.adjacency[node]:
				self.behind_around[neighbor] += step
				dirty.add(neighbor)

	def move(self, origin, target):
		dirty = set()
		self.shift(origin, -1, dirty)
		self.shift(target, 1, dirty)
		touched = set()
		for node in dirty:
			owner = self.owner_of(node)
			if owner != self.owners[node]:
				self.owners[node] = owner
				touched.update(self.node_triangles[node])
			penalty = self.penalty_of(node)
			self.value += penalty - self.penalties[node]
			self.penalties[node] = penalty
		for index in touched:
			value = self.triangle_value(index)
			self.value += value - self.triangle_values[index]
			self.triangle_values[index] = value


def nearest_target_field(turn, distances):
	targets = set()
	for corners, owner, me_can, _ in turn.triangles:
		if owner != 0 and me_can:
			targets.update(corners)
	if not targets:
		return [0] * len(turn.mine)
	return [min(row[t] for t in targets) for row in distances]


def assign_goals(turn, distances, units):
	owners = [
		node_owner(n, turn.mine, turn.theirs, turn.adjacency)
		for n in range(len(turn.mine))
	]
	will_own = {n for n, owner in enumerate(owners) if owner == 0}
	goals = [None] * len(units)
	free = set(range(len(units)))
	remaining = [
		(corners, owner)
		for corners, owner, me_can, _ in turn.triangles
		if owner != 0 and me_can
	]
	while free and remaining:
		best = None
		for index, (corners, owner) in enumerate(remaining):
			taken = []
			cost_sum = 0
			cost_max = 0
			for corner in corners:
				if corner in will_own:
					continue
				nearest = None
				for unit in free:
					if unit in taken:
						continue
					d = distances[units[unit]][corner]
					if nearest is None or d < nearest[0]:
						nearest = (d, unit)
				if nearest is None:
					break
				taken.append(nearest[1])
				cost_sum += nearest[0]
				cost_max = max(cost_max, nearest[0])
			else:
				cost = (
					cost_max
					+ 0.2 * cost_sum
					- (OPPONENT_TRIANGLE_BONUS if owner == 1 else 0)
				)
				if best is None or cost < best[0]:
					best = (cost, index, taken)
		if best is None:
			break
		_, index, taken = best
		corners, _ = remaining.pop(index)
		needed = [c for c in corners if c not in will_own]
		for corner in corners:
			if corner in will_own:
				for unit in list(free):
					if units[unit] == corner:
						goals[unit] = corner
						free.discard(unit)
						break
		order = sorted(needed, key=lambda c: min(distances[units[u]][c] for u in taken))
		pool = list(taken)
		for corner in order:
			unit = min(pool, key=lambda u: distances[units[u]][corner])
			pool.remove(unit)
			goals[unit] = corner
			free.discard(unit)
		will_own.update(needed)
	return goals


def first_step(origin, goal, adjacency, distances):
	if goal is None or goal == origin:
		return origin
	for next_node in sorted(adjacency[origin]):
		if distances[next_node][goal] < distances[origin][goal]:
			return next_node
	return origin


def plan_moves(turn, distances, deadline):
	nearest_target = nearest_target_field(turn, distances)
	units = []
	for node, count in enumerate(turn.mine):
		units.extend([node] * count)
	goals = assign_goals(turn, distances, units)

	def unit_cost(index, node):
		goal = goals[index]
		if goal is None:
			return IDLE_WEIGHT * nearest_target[node]
		return GOAL_WEIGHT * distances[node][goal]

	destinations = [
		first_step(origin, goal, turn.adjacency, distances)
		for origin, goal in zip(units, goals)
	]
	mine = [0] * len(turn.mine)
	for node in destinations:
		mine[node] += 1
	position = Position(turn, mine)
	goal_total = sum(unit_cost(i, node) for i, node in enumerate(destinations))
	best = position.value - goal_total
	improved = True
	while improved and time.time() < deadline:
		improved = False
		for index, origin in enumerate(units):
			if time.time() >= deadline:
				break
			current = destinations[index]
			for candidate in [origin] + turn.adjacency[origin]:
				if candidate == current:
					continue
				position.move(current, candidate)
				candidate_goal_total = (
					goal_total - unit_cost(index, current) + unit_cost(index, candidate)
				)
				value = position.value - candidate_goal_total
				if value > best + 1e-9:
					best = value
					goal_total = candidate_goal_total
					current = candidate
					improved = True
				else:
					position.move(candidate, current)
			destinations[index] = current
	moves = {}
	for origin, target in zip(units, destinations):
		if origin != target:
			moves[(origin, target)] = moves.get((origin, target), 0) + 1
	commands = [
		f"MOVE {origin} {target} {amount}" for (origin, target), amount in moves.items()
	]
	return position.value, commands


def plan_spawns(turn, distances):
	army = sum(turn.mine)
	owned = [corners for corners, owner, _, _ in turn.triangles if owner == 0]
	if army >= MAX_ARMY or not owned:
		return []
	frontier = set()
	for corners, owner, me_can, _ in turn.triangles:
		if owner != 0 and me_can:
			frontier.update(corners)
	if not frontier:
		return []

	def spawn_corner(corners):
		return min(corners, key=lambda n: min(distances[n][f] for f in frontier))

	ranked = sorted(
		owned,
		key=lambda corners: min(distances[spawn_corner(corners)][f] for f in frontier),
	)
	spawns = []
	for corners in ranked[: MAX_ARMY - army]:
		spawn = spawn_corner(corners)
		others = [n for n in corners if n != spawn]
		spawns.append(f"SPAWN {spawn} {others[0]} {others[1]}")
	return spawns


def material(turn, spawns):
	owned = sum(1 for _, owner, _, _ in turn.triangles if owner == 0)
	units = sum(turn.mine) - sum(turn.theirs) + len(spawns)
	return UNIT_VALUE * units + TRIANGLE_VALUE * (owned - len(spawns))


def attack_candidates(turn):
	candidates = []
	for index, (corners, owner, _, _) in enumerate(turn.triangles):
		if owner != 0 or any(turn.mine[c] < 1 for c in corners):
			continue
		targets = {
			n for c in corners for n in turn.adjacency[c] if turn.theirs[n] >= MIN_KILLS
		}
		for target in targets:
			candidates.append((turn.theirs[target], index, target))
	candidates.sort(reverse=True)
	return candidates[:MAX_ATTACK_CANDIDATES]


def value_of(turn, distances, deadline):
	move_value, moves = plan_moves(turn, distances, deadline)
	spawns = plan_spawns(turn, distances)
	return move_value + material(turn, spawns), moves + spawns


def value_of(turn, distances, deadline):
	move_value, moves = plan_moves(turn, distances, deadline)
	spawns = plan_spawns(turn, distances)
	return move_value + material(turn, spawns), moves + spawns


def plan(turn, distances, start):
	end = start + TIME_BUDGET
	best_value, best_commands = value_of(turn, distances, end)
	attacks = []
	state = turn
	while len(attacks) < MAX_ATTACKS and time.time() < end:
		chosen = None
		threshold = best_value + ATTACK_MARGIN
		for _, index, target in attack_candidates(state):
			if time.time() >= end:
				break
			after = state.after_attack(index, target)
			value, commands = value_of(after, distances, end)
			if value > threshold:
				threshold = value
				corners = state.triangles[index][0]
				attack = f"ATTACK {corners[0]} {corners[1]} {corners[2]} {target}"
				chosen = (value, commands, after, attack)
		if chosen is None:
			break
		best_value, best_commands, state, attack = chosen
		attacks.append(attack)
	return attacks + best_commands


def main():
	house_count = int(input())
	for _ in range(house_count):
		input()
	while True:
		turn = Turn(house_count)
		start = time.time()
		distances = bfs_all(turn.adjacency)
		commands = plan(turn, distances, start)
		print(";".join(commands) if commands else "WAIT", flush=True)


main()
