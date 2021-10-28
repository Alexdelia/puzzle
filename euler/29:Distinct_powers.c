/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   29:Distinct_powers.c                               :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/28 17:30:19 by adelille          #+#    #+#             */
/*   Updated: 2021/10/28 17:43:44 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdlib.h>
#include <math.h>

#define TRUE	1
#define FALSE	0

static int	ft_is_in(double d[10000], int size, double n)
{
	int	i;

	i = 0;
	while (i < size)
	{
		if (d[i] == n)
			return (TRUE);
		i++;
	}
	return (FALSE);
}

int	main(int ac, char **av)
{
	int		max;
	int		a;
	int		b;
	int		i;
	double	d[10000] = { -1.0 };
	double	tmp;

	if (ac == 2)
		max = atoi(av[1]);
	else
		max = 5;
	i = 0;
	a = 2;
	while (a <= max && i < 10000)
	{
		b = 2;
		while (b <= max && i < 10000)
		{
			tmp = pow(a, b);
			if (ft_is_in(d, i, tmp) == FALSE)
			{
				d[i] = tmp;
				i++;
			}
			b++;
		}
		a++;
	}
	printf("Distinct powers: %d\n", i);
	return (0);
}
