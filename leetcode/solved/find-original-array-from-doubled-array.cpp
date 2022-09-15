class Solution {
public:
    vector<int> findOriginalArray(vector<int> &changed)
	{
		if (changed.size() % 2)
			return {};

		vector<int>             original;
		unordered_map<int, int> count;

		for (int i : changed)
			count[i]++;

		sort(changed.begin(), changed.end());

		for (int i : changed)
        {
			if (!count[i])
				continue;
			if (!count[i << 1])
				return {};
			count[i]--;
			count[i << 1]--;
			original.push_back(i);
		}

		return original;
	}
};