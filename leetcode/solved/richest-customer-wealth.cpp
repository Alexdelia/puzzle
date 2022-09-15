class Solution {
public:
    int maximumWealth(const vector<vector<int>> &a)
	{
		int max = 0;
        for (auto x = a.begin(); x != a.end(); ++x)
		{
			unsigned short sum = 0;
			for (auto y = x->begin(); y != x->end(); ++y)
				sum += *y;
			if (sum > max)
				max = sum;
		}
		return max;
    }
};