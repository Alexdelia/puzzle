/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Vox_Codei_-_Episode_1.cpp                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/07/06 13:28:25 by adelille          #+#    #+#             */
/*   Updated: 2022/07/06 14:11:32 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#pragma GCC optimize("O3")

#include <iostream>
#include <string>
#include <vector>
#include <algorithm>

#define BOMB	64
#define WALL	35

int	w,h;
std::vector<std::vector<int>>	m;

static void	debug_print_map()
{
	std::cerr << w << ' ' << h << std::endl;

	for (int x = 0; x < h; x++)
	{
		for (int y = 0; y < w; y++)
			std::cerr << (m[x][y] == '@' ? '@' : (m[x][y] == '#' ? '#' : m[x][y])) << ' ';
		std::cerr << std::endl;
	}

}

static void	fill_map()
{
	m.resize(h);

	for (int i = 0; i < h; i++)
	{
		m[i].resize(w);

		std::string map_row;
		getline(std::cin, map_row); // one line of the firewall grid
		for (int x = 0; x < map_row.size(); x++)
		{
			if (map_row[x] == '.')
				m[i][x] = 0;
			else
				m[i][x] = map_row[x];
		}
	}
}

static void	update_map(const int bomb_x, const int bomb_y)
{
	int	x, y;

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
	
	for (x = bomb_x; x < w && m[x][bomb_y] != '#'; x++)
		if (m[x][bomb_y] == '@')
			m[x][bomb_y] = 3;
	for (y = bomb_y; y < h && m[bomb_x][y] != '#'; y++)
		if (m[bomb_x][y] == '@')
			m[bomb_x][y] = 3;
	for (x = bomb_x; x >= 0 && m[x][bomb_y] != '#'; x--)
		if (m[x][bomb_y] == '@')
			m[x][bomb_y] = 3;
	for (y = bomb_y; y >= 0 && m[bomb_x][y] != '#'; y--)
		if (m[bomb_x][y] == '@')
			m[bomb_x][y] = 3;
}

static unsigned int	calc_score(const int base_x, const int base_y)
{
	int	x, y;
	unsigned int	score = 0;

	for (x = base_x; x < w && m[x][base_y] != '#'; x++)
		if (m[x][base_y] == '@')
			score++;
	for (y = base_y; y < h && m[base_x][y] != '#'; y++)
		if (m[base_x][y] == '@')
			score++;
	for (x = base_x; x >= 0 && m[x][base_y] != '#'; x--)
		if (m[x][base_y] == '@')
			score++;
	for (y = base_y; y >= 0 && m[base_x][y] != '#'; y--)
		if (m[base_x][y] == '@')
			score++;
	
	return (score);
}

static void	solve(int *best_x, int *best_y)
{
	int	x, y;
	unsigned short	best_score = 0;
	unsigned short	score = 0;

	x = 0;
	while (x < h)
	{
		y = 0;
		while (y < w)
		{
			score = calc_score(x, y);
			if (score > best_score)
			{
				best_score = score;
				*best_x = x;
				*best_y = y;
			}
			y++;
		}
		x++;
	}
}

int	main(void)
{
	std::cin >> w >> h; std::cin.ignore();
	fill_map();
	debug_print_map();

	// game loop
	while (1) {
		int r; // number of rounds left before the end of the game
		int b; // number of bombs left
		std::cin >> r >> b; std::cin.ignore();

		int	x = 0, y = 0;
		solve(&x, &y);
		update_map(x, y);

		std::cout << x << ' ' << y << std::endl;
	}

	return (0);
}
