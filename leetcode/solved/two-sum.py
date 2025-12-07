class Solution:
	def twoSum(self, nums: list[int], target: int) -> list[int]:  # noqa: N802
		d = {}
		for i in range(len(nums)):
			if target - nums[i] in d:
				return [d[target - nums[i]], i]

			d[nums[i]] = i

		raise ValueError("No two sum solution")
