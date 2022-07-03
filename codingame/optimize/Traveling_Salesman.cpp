/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Traveling_Salesman.cpp                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/07/03 15:54:06 by adelille          #+#    #+#             */
/*   Updated: 2022/07/03 18:55:31 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#pragma GCC optimize("O3")

#include <iostream>
#include <string>
#include <vector>
#include <cmath>
#include <bits/stdc++.h>

#define DEBUG	1

#define DEPTH	1

typedef unsigned short	t_coord;
typedef unsigned short	t_index;

typedef struct s_coordinate
{
	t_coord	x;
	t_coord	y;
	bool	visited;
}			t_coordinate;

std::vector<t_coordinate>						m;	// all destination
std::map<std::pair<t_index, t_index>, float>	ad;	// all distance
std::vector<t_index>							p;	// final path
float											best_d;	// best distance

static float	distance(const t_index one, const t_index two)
{
	return (sqrtf(powf((m[two].x - m[one].x), 2) + powf((m[two].y - m[one].y), 2)));
}

/*static float	distance(const int x1, const int y1, const int x2, const int y2)
{
	return (sqrtf(powf((x2 - x1), 2) + powf((y2 - y1), 2)));
}*/

static const std::map<float, int>	&distance(const t_index curr, std::map<float, int> &dist)
{
	t_index					i = 0;

	while (i < m.size())
	{
		if (m[i].visited == false)
			dist[ad[std::make_pair(curr, i)]] = i;
		i++;
	}

	return (dist);
}

static void	all_distance(void)
{
	t_index			one;
	t_index			two;
	const t_index	size = m.size();

	one = 0;
	while (one < size)
	{
		two = 0;
		while (two < size)
		{
			ad[std::make_pair(one, two)] = distance(one, two);
			two++;
		}
		one++;
	}	
}

static void	solve(const t_index curr, const int node, const float t, std::vector<t_index> &path)
{
	/*if (DEBUG)
	{
		std::cerr << "> ";
		for (std::vector<t_index>::const_iterator i = path.begin(); i != path.end(); ++i)
			std::cerr << *i << ' ';
		std::cerr << '\t' << t << '\t' << node << '\t' << curr << std::endl;
	}*/

	if (t >= best_d)
		return ;

	if (node == 0)
	{
		if (t + ad[std::make_pair(curr, 0)] < best_d)
		{
			best_d = t + ad[std::make_pair(curr, 0)];
			p = path;
			if (DEBUG)
			{
				for (std::vector<t_index>::const_iterator i = p.begin(); i != p.end(); ++i)
					std::cerr << *i << ' ';
				std::cerr << "0\t" << best_d << std::endl;
			}
		}
		return ;
	}

	std::map<float, int>	dist;
	distance(curr, dist);
	int	depth = 0;
	for (std::map<float, int>::const_iterator i = dist.begin(); i != dist.end() && depth < DEPTH; ++i, depth++)
	{
		m[i->second].visited = true;
		path.push_back(i->second);
		solve(i->second, node - 1, t + i->first, path);
		m[i->second].visited = false;
		path.pop_back();
	}
	return ;
}

int	main(void)
{
	int	n; // This variables stores how many nodes are given
	std::cin >> n; std::cin.ignore();
	for (int i = 0; i < n; i++)
	{
		t_coordinate	tmp;
	    std::cin >> tmp.x >> tmp.y; std::cin.ignore();
		tmp.visited = false;
		m.push_back(tmp);
	}

	best_d = 2000 * n;
	all_distance();

	std::vector<t_index>	path(1, 0);
	m[0].visited = true;
	solve(0, n - 1, 0, path);

	for (std::vector<t_index>::const_iterator i = p.begin(); i != p.end(); ++i)
		std::cout << *i << ' ';
	std::cout << '0' << std::endl;
	std::cerr << best_d << std::endl;
	
	return (0);
}