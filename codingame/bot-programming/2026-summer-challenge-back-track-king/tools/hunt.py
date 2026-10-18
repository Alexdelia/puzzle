#!/usr/bin/env python3
import json, os, re, shutil, subprocess, sys, tempfile, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "target", "release")
PIN = ["env", "BTK_THINK_MS=3000", "BTK_PLOTS=0"]
RANGES = [7000000, 7100000]
GAMES = int(os.environ.get("HUNT_GAMES", "200"))

snap = tempfile.mkdtemp(prefix="hunt.")
shutil.copy(
	os.path.join(BIN, os.environ.get("HUNT_BOT", "delta")), os.path.join(snap, "bot")
)
FOES = []
for name in os.environ.get("HUNT_FOES", "sentinel").split(","):
	shutil.copy(os.path.join(BIN, name), os.path.join(snap, name))
	FOES.append(" ".join(PIN + [os.path.join(snap, name)]))
BOT = os.path.join(snap, "bot")

WIN = re.compile(r"^p1 wins: (\d+) ")
PTS = re.compile(r"^mean points: p1 ([\d.]+)  p2 ([\d.]+)")


def evaluate(env, ranges=RANGES):
	spec = " ".join(PIN + [f"{k}={v}" for k, v in sorted(env.items())] + [BOT])
	wins = total = 0
	mine = yours = 0.0
	per = []
	for foe in FOES:
		for seed in ranges:
			out = subprocess.run(
				[
					os.path.join(BIN, "arena"),
					"--games",
					str(GAMES),
					"--seed",
					str(seed),
					"--quiet",
					"--no-timings",
					"--p1",
					spec,
					"--p2",
					foe,
				],
				capture_output=True,
				text=True,
			).stdout
			w = int(
				next(WIN.match(l).group(1) for l in out.splitlines() if WIN.match(l))
			)
			a, b = next(
				(float(m.group(1)), float(m.group(2)))
				for l in out.splitlines()
				if (m := PTS.match(l))
			)
			wins += w
			total += GAMES
			mine += a
			yours += b
			per.append(100.0 * w / GAMES)
	return 100.0 * wins / total, 100.0 * mine / (mine + yours), per


def label(env):
	return " ".join(f"{k}={v}" for k, v in sorted(env.items())) or "<base>"


def main():
	plan = json.load(open(sys.argv[1]))
	env = dict(plan["base"])
	best_win, best_pts, per = evaluate(env)
	print(
		f"base {label(env)}\n  {best_win:.1f}% win  {best_pts:.2f}% pts  {per}",
		flush=True,
	)
	for rnd in range(plan.get("rounds", 1)):
		moved = False
		for knob, values in plan["knobs"].items():
			for value in values:
				if str(env.get(knob, "")) == str(value):
					continue
				trial = dict(env)
				if value == "":
					trial.pop(knob, None)
				else:
					trial[knob] = value
				win, pts, per = evaluate(trial)
				flag = ""
				if win > best_win + plan.get("margin", 0.75):
					best_win, best_pts, env, moved = win, pts, trial, True
					flag = "  <== kept"
				print(
					f"  {knob}={value:<8} {win:5.1f}% win  {pts:5.2f}% pts  {per}{flag}",
					flush=True,
				)
		print(f"round {rnd}: {best_win:.1f}%  {label(env)}", flush=True)
		if not moved:
			break
	print(f"FINAL {best_win:.1f}% win  {best_pts:.2f}% pts\n{label(env)}", flush=True)


main()
