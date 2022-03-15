/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   lili_math_problem.c                                :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/08 16:10:43 by adelille          #+#    #+#             */
/*   Updated: 2022/03/14 16:35:03 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

//#include "../libft/inc/libft.h"
#include <stdio.h>

#define MIN	1000
#define MAX	2000000000

/*
	x is smaller than INT_MAX, but will use size_t anyway
	v max is 72
*/

static size_t	ft_pow(size_t n, size_t p)
{
	size_t	base;

	base = n;
	while (p > 1)
	{
		n *= base;
		p--;
	}
	return (n);
}

int	main(void)
{
	size_t	x;
	size_t	y;
	size_t	z;
	size_t	v;

	y = 2;
	z = 2;
	v = 2;

	while (ft_pow(y, 2) < MIN)
		y++;
	while (ft_pow(y, 2) < MAX)
	{
		x = ft_pow(y, 2);
		while (x > ft_pow(z, 3))
			z++;
		if (x == ft_pow(z, 3))
		{
			while (x > ft_pow(v, 5))
				v++;
			if (x == ft_pow(z, 3) && x == ft_pow(v, 5))
				printf("\033[2K\rx=%ld\t(y=%ld, z=%ld, v=%ld)\n", x, y, z, v);
		}
		printf("\r%ld: %ld, %ld, %ld", x, y, z, v);
		y++;
	}
	printf("\n");
	return (0);
}
