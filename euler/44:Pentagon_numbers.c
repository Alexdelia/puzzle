/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   44:Pentagon_numbers.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/08 16:10:43 by adelille          #+#    #+#             */
/*   Updated: 2022/02/08 16:29:57 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../libft/inc/libft.h"
#include <stdio.h>

#define SIZE	10000

static bool	ft_is_pentagonal(long x, unsigned long *p)
{
	size_t	i;

	if (x < 1 || x > p[SIZE - 1])
		return (false);
	i = 1;
	while (i < SIZE && p[i] < x)
		++i;
	if (p[i] == x)
		return (true);
	return (false);
}

int	main(void)
{
	unsigned long	p[SIZE];
	size_t			x;
	size_t			y;
	size_t			i;

	p[0] = 0;
	i = 0;
	while (++i < SIZE)
		p[i] = i * (3 * i - 1) / 2;
	printf("%ld\n", p[SIZE - 1]);
	x = 1;
	while (x < SIZE)
	{
		printf("\r[%ld%%]", x / SIZE * 100);
		y = 1;
		while (y < SIZE)
		{
			if (ft_is_pentagonal(p[x] + p[y], p) && ft_is_pentagonal(p[y] - p[x], p))
			{
				printf("P%ld - P%ld = %ld - %ld = %ld", y, x, p[y], p[x], p[y] - p[x]);
				return (0);
			}
			y++;
		}
		x++;
	}
	return (0);
}
