/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Traveling_Salesman.cpp                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/07/03 15:54:06 by adelille          #+#    #+#             */
/*   Updated: 2022/07/03 19:54:47 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#pragma GCC optimize("O3")

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <cmath>
#include <bits/stdc++.h>

#define DEPTH       2
#define MAX_DIST    1000

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

static const float	distance(const t_index one, const t_index two)
{
	if (ad.find(std::make_pair(one, two)) == ad.end())
		ad.insert(std::make_pair(std::make_pair(one, two), sqrtf(powf((m[two].x - m[one].x), 2) + powf((m[two].y - m[one].y), 2))));
	return (ad[std::make_pair(one, two)]);
}

static const std::map<float, int>	&distance(const t_index curr, std::map<float, int> &dist)
{
	t_index					i = 0;

	while (i < m.size())
	{
		if (m[i].visited == false)
			dist[distance(curr, i)] = i;
		i++;
	}

	return (dist);
}

static void	solve(const t_index curr, const int node, const float t, std::vector<t_index> &path)
{
    if (t > best_d)
        return ;

	if (node == 0)
	{
		if (t + distance(curr, 0) < best_d)
		{
			best_d = t + distance(curr, 0);
			p = path;
		}
		return ;
	}

	std::map<float, int>	dist;
	distance(curr, dist);
	int	depth = 0;
	for (std::map<float, int>::const_iterator i = dist.begin();
            i != dist.end() && depth < DEPTH && i->second < MAX_DIST; ++i)
	{
        m[i->second].visited = true;
		path.push_back(i->second);
		solve(i->second, node - 1, t + i->first, path);
		m[i->second].visited = false;
		path.pop_back();
        depth++;
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

	std::vector<t_index>	path(1, 0);
	m[0].visited = true;
	solve(0, n - 1, 0, path);

	for (std::vector<t_index>::const_iterator i = p.begin(); i != p.end(); ++i)
		std::cout << *i << ' ';
	std::cout << '0' << std::endl;
	std::cerr << best_d << std::endl;

	return (0);
}