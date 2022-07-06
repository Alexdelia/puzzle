/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Vox_Codei_-_Episode_1.cpp                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/07/06 13:28:25 by adelille          #+#    #+#             */
/*   Updated: 2022/07/06 15:49:08 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#pragma GCC optimize("O3")

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <set>
#include <algorithm>

int	h, w;
std::vector<std::vector<int>>					m;
std::set<std::pair<int, int>>					bombs;
std::map<std::pair<int, int>, unsigned short>	scores;

static void	debug_print_map()
{
	std::cerr << "height:\t" << h << std::endl
		<< "wide:\t" << w << std::endl;

	for (int x = 0; x < h; x++)
	{
		for (int y = 0; y < w; y++)
		{
			if (m[x][y] == '@')
				std::cerr << "@ ";
			else if (m[x][y] == '#')
				std::cerr << "# ";
			else
				std::cerr << m[x][y] << ' ';
		}
		std::cerr << std::endl;
	}

}

static void	fill_map()
{
	m.resize(h);

	for (int x = 0; x < h; x++)
	{
		m[x].resize(w);

		std::string map_row;
		getline(std::cin, map_row); // one line of the firewall grid
		for (int y = 0; y < map_row.size(); y++)
		{
			if (map_row[y] == '.')
				m[x][y] = 0;
			else
			{
				if (map_row[y] == '@')
					bombs.insert(std::make_pair(x, y));
				m[x][y] = map_row[y];
			}
		}
	}
}

static void	update_map(const int bomb_x, const int bomb_y)
{
	int	x, y;
	
	scores.clear();
	
	// update all timed explosion
	x = 0;
	while (x < h)
	{
		y = 0;
		while (y < w)
		{
			if (m[x][y] > 0 && m[x][y] <= 3)
				m[x][y] -= 1;
			y++;
		}
		x++;
	}

	// place current bomb
	m[bomb_x][bomb_y] = 3;
	for (x = bomb_x; x < h && x <= bomb_x + 3 && m[x][bomb_y] != '#'; x++)
	{
		if (m[x][bomb_y] == '@')
		{
			m[x][bomb_y] = 3;
			bombs.erase(std::make_pair(x, bomb_y));
		}
	}
	for (y = bomb_y; y < w && y <= bomb_y + 3 && m[bomb_x][y] != '#'; y++)
	{
		if (m[bomb_x][y] == '@')
		{
			m[bomb_x][y] = 3;
			bombs.erase(std::make_pair(bomb_x, y));
		}
	}
	for (x = bomb_x; x >= 0 && x >= bomb_x - 3 && m[x][bomb_y] != '#'; x--)
	{
		if (m[x][bomb_y] == '@')
		{
			m[x][bomb_y] = 3;
			bombs.erase(std::make_pair(x, bomb_y));
		}
	}
	for (y = bomb_y; y >= 0 && y >= bomb_y - 3 && m[bomb_x][y] != '#'; y--)
	{
		if (m[bomb_x][y] == '@')
		{
			m[bomb_x][y] = 3;
			bombs.erase(std::make_pair(bomb_x, y));
		}
	}
}

static unsigned int	calc_score(const int base_x, const int base_y)
{
	const std::pair<int, int>	p = std::make_pair(base_x, base_y);

	if (scores.find(p) != scores.end())
		return (scores[p]);
	
	int				x, y;
	unsigned short	score = 0;

	for (x = base_x; x < h && x <= base_x + 3 && m[x][base_y] != '#'; x++)
		if (m[x][base_y] == '@')
			score++;
	for (y = base_y; y < w && y <= base_y + 3 && m[base_x][y] != '#'; y++)
		if (m[base_x][y] == '@')
			score++;
	for (x = base_x; x >= 0 && x >= base_x - 3 && m[x][base_y] != '#'; x--)
		if (m[x][base_y] == '@')
			score++;
	for (y = base_y; y >= 0 && y >= base_y - 3 && m[base_x][y] != '#'; y--)
		if (m[base_x][y] == '@')
			score++;

	scores.insert(std::make_pair(p, score));

	return (score);
}

static void	try_score(const int x, const int y, unsigned short *best_score, int *best_x, int *best_y)
{
	const unsigned short	score = calc_score(x, y);

	if (score > *best_score)
	{
		*best_score = score;
		*best_x = x;
		*best_y = y;
	}
}

static void	solve(int *best_x, int *best_y)
{
	std::set<std::pair<int, int>>::const_iterator	i = bombs.begin();
	unsigned short	best_score = 0;
	int				x, y;

	while (i != bombs.end())
	{
		// try to see if score around bomb is > best_score
		for (x = i->first; x < h && x <= i->first + 3 && m[x][i->second] != '#'; x++)
			if (m[x][i->second] == 0)
				try_score(x, i->second, &best_score, best_x, best_y);
		for (y = i->second; y < w && y <= i->second + 3 && m[i->first][y] != '#'; y++)
			if (m[i->first][y] == 0)
				try_score(i->first, y, &best_score, best_x, best_y);
		for (x = i->first; x >= 0 && x >= i->first - 3 && m[x][i->second] != '#'; x--)
			if (m[x][i->second] == 0)
				try_score(x, i->second, &best_score, best_x, best_y);
		for (y = i->second; y >= 0 && y >= i->second - 3 && m[i->first][y] != '#'; y--)
			if (m[i->first][y] == 0)
				try_score(i->first, y, &best_score, best_x, best_y);
		++i;
	}
}

int	main(void)
{
	std::cin >> w >> h; std::cin.ignore();
	fill_map();

	// game loop
	while (1) {
		int r; // number of rounds left before the end of the game
		int b; // number of bombs left
		std::cin >> r >> b; std::cin.ignore();

		if (!bombs.empty())
		{
			int	x = 0, y = 0;
			solve(&x, &y);
			std::cerr << "score:\t" << calc_score(x, y) << std::endl;
			update_map(x, y);
			
			debug_print_map();

			std::cout << y << ' ' << x << std::endl;
		}
		else
			std::cout << "WAIT" << std::endl;
	}

	return (0);
}
