/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   28:Number_spiral_diagonals.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/28 17:12:11 by adelille          #+#    #+#             */
/*   Updated: 2021/10/28 17:25:52 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdlib.h>

int	main(int ac, char **av)
{
	int		side;
	int		i;
	int		s;
	int		four;
	long	sum;

	if (ac == 2)
		side = atoi(av[1]);
	else
		side = 1001;
	i = 1;
	s = 2;
	sum = 1;
	while (s <= side)
	{
		four = 0;
		while (four < 4)
		{
			i += s;
			sum += i;
			four++;
		}
		s += 2;
	}
	printf("sum for square of %dx%d: %ld\n", side, side, sum);
	return (0);
}
