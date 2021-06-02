/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_formate.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/31 17:52:23 by adelille          #+#    #+#             */
/*   Updated: 2021/05/31 19:45:27 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "fo.h"

static char	ft_big_switch_basic(char c)
{
	// the switch of death
	switch (c)
	{
		case ('o'):
			return ('0');
		case ('O'):
			return ('0');
		case ('i'):
			return ('1');
		case ('I'):
			return ('1');
		case ('z'):
			return ('2');
		case ('Z'):
			return ('2');
		case ('e'):
			return ('3');
		case ('E'):
			return ('3');
		case ('a'):
			return ('4');
		case ('A'):
			return ('4');
		case ('s'):
			return ('5');
		case ('S'):
			return ('5');
		case ('t'):
			return ('7');
		case ('T'):
			return ('7');
		case ('b'):
			return ('8');
		case ('B'):
			return ('8');
		case ('g'):
			return ('9');
		case ('G'):
			return ('9');
	}
	return (c);
}

static char	ft_big_switch_uwu(char c)
{
	// the switch of sowwow uwu
	switch (c)
	{
		case ('l'):
			return ('w');
		case ('L'):
			return ('W');
		case ('r'):
			return ('w');
		case ('R'):
			return ('W');
		case ('s'):
			return ('z');
		case ('S'):
			return ('Z');
		case ('v'):
			return ('W');
		case ('V'):
			return ('W');
	}
	return (c);
}

static int	ft_nal(char c)
{
	if ((c >= 'A' && c <= 'Z')
			|| (c >= 'a' && c <= 'z'))
		return (TRUE);
	return (FALSE);
}

static int	ft_uwu_intensity(int x)
{
	int	p[11] = {1000, 100, 50, 20, 10, 7, 5, 4, 3, 2, 1};
	return (p[x]);
}

void	ft_formate(t_arg *a)
{
	int		i;
	char	(*f[2])(char);
	
	i = 0;
	f[DEFAULT] = ft_big_switch_basic;
	f[UWU] = ft_big_switch_uwu;
	if (a->type == UWU)
		srand(time(NULL));
	while (a->str[i])
	{
		if (a->type == UWU && (i == 0 || ft_nal(a->str[i - 1]) == FALSE))
			a->res[i] = a->str[i];
		else if (a->type == UWU && rand() % ft_uwu_intensity(a->intensity) == 0 && ft_isalpha((int)a->str[i]))
			a->res[i] = 'w';
		else
			a->res[i] = f[a->type](a->str[i]);
		i++;
	}
}
