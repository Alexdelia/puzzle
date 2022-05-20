/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Code4Life.cpp                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/05/20 11:40:48 by adelille          #+#    #+#             */
/*   Updated: 2022/05/20 15:49:06 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#pragma GCC optimize("O3")

#include <iostream>
#include <string>
#include <sstream>
#include <ostream>
#include <vector>
#include <map>
#include <algorithm>

#define DEBUG	0

#define GO	"GOTO "
#define CO	"CONNECT "

#define STATUS_DIA	0
#define STATUS_MOL	1
#define STATUS_LAB	2

typedef struct project
{
	int a;
	int b;
	int c;
	int d;
	int e;
} s_p;

static int get_project(s_p *p)
{
	int project_count;
	std::cin >> project_count;
	std::cin.ignore();

	if (project_count != 0)
		std::cerr << "project_count = " << project_count << std::endl;

	for (int i = 0; i < project_count; i++)
		std::cin >> p->a >> p->b >> p->c >> p->d >> p->e;
	std::cin.ignore();

	if (DEBUG && project_count != 0)
		std::cerr << p->a << ' ' << p->b << ' ' << p->c << ' ' << p->d << ' ' << p->e << std::endl;

	return (project_count);
}

class sample
{
	public:
		sample() {};
		~sample() {};
		
		int			sample_id;
		int			carried_by;
		int			rank;
		std::string	expertise_gain;
		int			health;
		int			cost_a;
		int			cost_b;
		int			cost_c;
		int			cost_d;
		int			cost_e;
};

std::stringstream	&operator<<(std::stringstream &o, const sample &src)
{
	o << src.sample_id << "\t| " << src.carried_by << ' ' << src.rank << ' ' << src.expertise_gain << ' ' << src.health
		<< "\tcost: " << src.cost_a << ' ' << src.cost_b << ' ' << src.cost_c << ' ' << src.cost_d << ' ' << src.cost_e
		<< std::endl;
	return (o);
}

class robot;

class room
{
	public:
		room() {};
		~room() {};

		static const std::string	dia;
		static const std::string	mol;
		static const std::string	lab;
		
		int		available_a;
		int		available_b;
		int		available_c;
		int		available_d;
		int		available_e;

		std::map<int, sample>	samples;

		void	fill_data(void);
		void	get_samples(void);

		std::string	print_samples(void) const;
		int			find_best_sample_id(void) const;
		void		action_cost_of_id(robot &I, const int id);
};

const std::string	room::dia = "DIAGNOSIS";
const std::string	room::mol = "MOLECULES";
const std::string	room::lab = "LABORATORY";

std::ostream	&operator<<(std::ostream &o, const room &src)
{
	o << src.available_a << ' ' << src.available_b << ' ' << src.available_c << ' ' << src.available_d << ' ' << src.available_e
		<< src.print_samples();
	return (o);
}

std::string	room::print_samples(void) const
{
	std::stringstream						ret;
	std::map<int, sample>::const_iterator	i = samples.begin();

	while (i != samples.end())
	{
		ret << i->second << std::endl;
		++i;
	}
	return (ret.str());
}

void	room::fill_data(void)
{
	std::cin >> available_a >> available_b >> available_c >> available_d >> available_e;
	std::cin.ignore();
}

void	room::get_samples(void)
{
	int	sample_count;
	std::cin >> sample_count;
	std::cin.ignore();
	for (int i = 0; i < sample_count; i++)
	{
		sample	s;
		std::cin >> s.sample_id >> s.carried_by >> s.rank >> s.expertise_gain >> s.health
			>> s.cost_a >> s.cost_b >> s.cost_c >> s.cost_d >> s.cost_e;
		std::cin.ignore();

		this->samples[s.sample_id] = s;
	}
}

class robot
{
	public:
		robot() {};
		~robot() {};

		std::string	target;
		int			eta;
		int			score;
		int			storage_a;
		int			storage_b;
		int			storage_c;
		int			storage_d;
		int			storage_e;
		int			expertise_a;
		int			expertise_b;
		int			expertise_c;
		int			expertise_d;
		int			expertise_e;

		int							status;
		std::vector<std::string>	actions;

		void				fill_data(void);
		const std::string	action(room &r);
		void				find_action(room &r, const unsigned short n);
		void				print_cost_mol(const std::string &mol, const size_t time);
};

std::ostream	&operator<<(std::ostream &o, const robot &src)
{
	o << src.target << ' ' << src.eta << ' ' << src.score
		<< " storage: "
		<< src.storage_a << ' ' << src.storage_b << ' ' << src.storage_c << ' ' << src.storage_d << ' ' << src.storage_e
		<< " expertise: "
		<< src.expertise_a << ' ' << src.expertise_b << ' ' << src.expertise_c << ' ' << src.expertise_d << ' ' << src.expertise_e
		<< std::endl;
	return (o);
}

void	robot::fill_data(void)
{
	std::cin >> target >> eta >> score
		>> storage_a >> storage_b >> storage_c >> storage_d >> storage_e
		>> expertise_a >> expertise_b >> expertise_c >> expertise_d >> expertise_e;
	std::cin.ignore();
}

void	robot::print_cost_mol(const std::string &mol, const size_t time)
{
	size_t	i;

	i = 0;
	while (i < time)
	{
		actions.push_back(CO + mol);
		i++;
	}
}

void	room::action_cost_of_id(robot &I, const int id)
{
	I.print_cost_mol("A", samples[id].cost_a);
	I.print_cost_mol("B", samples[id].cost_b);
	I.print_cost_mol("C", samples[id].cost_c);
	I.print_cost_mol("D", samples[id].cost_d);
	I.print_cost_mol("E", samples[id].cost_e);
}

const std::string	robot::action(room &r)
{
	if (actions.empty())
		find_action(r, 3);
	
	const std::string	ret = *actions.begin();
	actions.erase(actions.begin());
	return (ret);
}

int	room::find_best_sample_id(void) const
{
	std::map<int, sample>::const_iterator	i = samples.begin();
	int	perf = -100;
	int	id = -1;
	int	tmp;

	while (i != samples.end())
	{
		if (i->second.carried_by == -1)	// test if can be picked up
		{
			tmp = i->second.health - (i->second.cost_a + i->second.cost_b + i->second.cost_c + i->second.cost_d + i->second.cost_e);
			if (tmp > perf)
			{
				perf = tmp;
				id = i->second.sample_id;
			}
		}
		++i;
	}

	return (id);
}

void	robot::find_action(room &r, const unsigned short n)
{
	std::vector<int>			id;
	std::vector<std::string>	s_id;
	unsigned short				i;

	actions.push_back(GO + room::dia);

	i = 0;
	while (i < n)
	{
		id.push_back(r.find_best_sample_id());
		s_id.push_back(std::to_string(id.back()));
		actions.push_back(CO + s_id.back());
		r.samples[id.back()].carried_by = -2;
		i++;
	}
	
	actions.push_back(GO + room::mol);
	i = 0;
	while (i < n)
	{
		r.action_cost_of_id(*this, id[i]);
		i++;
	}

	actions.push_back(GO + room::lab);
	i = 0;
	while (i < n)
	{
		actions.push_back(CO + s_id[i]);
		i++;
	}
}

int main(void)
{
	s_p		p;
	room	r;
	robot	I;
	robot	X;

	get_project(&p);

	I.status = -1;

	// game loop
	while (1)
	{
		I.fill_data();
		X.fill_data();
		if (DEBUG)
			std::cerr << "I: " << I << "X: " << X;

		r.fill_data();
		r.get_samples();
		if (DEBUG)
			std::cerr << r;

		std::cout << I.action(r) << std::endl;
	}

	return (0);
}