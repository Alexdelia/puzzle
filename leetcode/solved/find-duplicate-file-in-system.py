class Solution:
	def findDuplicate(self, paths: list[str]) -> list[list[str]]:  # noqa: N802
		d = {}
		for p in paths:
			ps = p.split()
			for f in ps[1:]:
				fs = f.split("(")
				fs[1] = fs[1][:-1]
				if fs[1] in d:
					d[fs[1]].append(ps[0] + "/" + fs[0])
				else:
					d[fs[1]] = [ps[0] + "/" + fs[0]]
		return [v for v in d.values() if len(v) > 1]
