class Solution:
	def fizzBuzz(self, n: int) -> list[str]:  # noqa: N802
		r = []
		i = 1
		while i <= n:
			if i % 15 == 0:
				r.append("FizzBuzz")
			elif i % 3 == 0:
				r.append("Fizz")
			elif i % 5 == 0:
				r.append("Buzz")
			else:
				r.append(str(i))
			i += 1
		return r
