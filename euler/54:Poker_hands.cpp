/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   54:Poker_hands.cpp                                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2022/04/06 23:01:55 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <iostream>
#include <vector>
#include <fstream>
#include <string>
#include <algorithm>

using namespace std;

static size_t	convert(const char &c)
{
	if (c <= 9)
		return (c - '0');
	else if (c == 'T')
		return (10);
	else if (c == 'J')
		return (11);
	else if (c == 'Q')
		return (12);
	else if (c == 'K')
		return (13);
	else if (c == 'A')
		return (14);
	else
	{
		cerr << "Error in file, \"" << c << "\" not recognised as a card" << endl;
		return (0); // I do not exit the program
	}
}

static void	parse(const string &line, vector<size_t> hand1, vector<size_t> hand2)
{
	size_t	i;

	i = 0;
	// need to get suit
	while (i < line.size())
	{
		if (i < 15) // line.size() / 2
			hand1.push_back(convert(line[i]));
		else
			hand2.push_back(convert(line[i]));
		i += 3;
	}
	sort(hand1.begin(), hand1.end());
	sort(hand2.begin(), hand2.end());
}

static bool	solve(const vector<size_t> &hand1, const vector<size_t> &hand2)
{

	return (true);
}

int	main(int ac, char **av)
{
	size_t			win;
	vector<size_t>	hand1(5);
	vector<size_t>	hand2(5);
	fstream			file;
	string			line;

	if (ac < 2)
		file.open("input:54.txt", ios::in);
	else
		file.open(av[1], ios::in);
	if (!file.is_open())
		return (cerr << "File error" << endl, 1);

	win = 0;
	while (getline(file, line))
	{
		if (line.size() != 29)
			return (cerr << "File dones't have expected format" << endl, 2);
		hand1.clear();
		hand2.clear();
		parse(line, hand1, hand2);
		win += solve(hand1, hand2);
	}
	file.close();
	cout << win << endl;
	return (0);
}
