#include <vector>
#include <map>
#include <climits>

class Solution
{
private:
	std::vector<std::vector<int> >	dp;
	const std::vector<int>	*n;
	const std::vector<int>	*m;
	unsigned int		n_len;
	unsigned int		m_len;

public:
    int maximumScore(const std::vector<int> &n, const std::vector<int> &m)
	{
		this->n = &n;
		this->m = &m;
		n_len = n.size();
		m_len = m.size();

		dp.resize(m_len + 1, std::vector<int>(m_len + 1, INT_MIN));

		return (recursive_maximum_score(0, 0));
    }
	
	int recursive_maximum_score(const unsigned int left, const unsigned int op)
	{
		if (op == m_len)
			return (0);

		if (dp[left][op] != INT_MIN)
			return (dp[left][op]);

		return (dp[left][op] = std::max(
			recursive_maximum_score(left + 1, op + 1) + (*n)[left] * (*m)[op],
			recursive_maximum_score(left, op + 1) + (*n)[n_len - (op - left) - 1] * (*m)[op]));
	}
};