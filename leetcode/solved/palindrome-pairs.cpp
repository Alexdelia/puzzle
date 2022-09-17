#include <string>
#include <vector>
#include <unordered_map>
#include <algorithm>
#include <climits>

struct TrieNode
{
    TrieNode			*next[26] = {};
    int					index = -1;
    std::vector<int>	p_index;
};

class Solution
{
private:
	TrieNode	root;

	void	add(const std::string &s, int i)
	{
		TrieNode	*node = &root;

		for (int x = s.size() - 1; x >= 0; --x)
		{
			if (is_palindrome(s, 0, x))
				node->p_index.push_back(i);
			const int	c = s[x] - 'a';
			if (!node->next[c])
				node->next[c] = new TrieNode();
			node = node->next[c];	
		}

		node->index = i;
		node->p_index.push_back(i);
	}

	bool	is_palindrome(const std::string &s, int x, int y)
	{
		if (s.size() < 2)
			return (true);

		while (x < y)
			if (s[x++] != s[y--])
				return (false);
		return (true);
	}

public:
    std::vector<std::vector<int> >	palindromePairs(const std::vector<std::string> &w)
	{
		std::vector<std::vector<int> >	res;
		int								x, y, size;

		size = w.size();

		if (size < 2)
			return (res);

		for (x = 0; x < size; ++x)
			add(w[x], x);

		for (x = 0; x < size; ++x)
		{
			TrieNode	*node = &root;

			for (y = 0; y < w[x].size() && node; ++y)
			{
				if (node->index != -1 && node->index != x && is_palindrome(w[x], y, w[x].size() - 1))
					res.push_back({x, node->index});
				node = node->next[w[x][y] - 'a'];
			}

			if (!node)
				continue;

			for (const int &i : node->p_index)
				if (i != x)
					res.push_back({x, i});
		}

		return (res);
    }
};