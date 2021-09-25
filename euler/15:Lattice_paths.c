/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   15:Lattice_paths.c                                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/24 20:28:50 by adelille          #+#    #+#             */
/*   Updated: 2021/09/25 08:38:12 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define	SIDE	21

// https://en.wikipedia.org/wiki/Central_binomial_coefficient
// https://www.youtube.com/watch?v=gMlf1ELvRzc

int	main(void)
{

	long	grid[SIDE][SIDE];
	int		x;
	int		y;

	x = 0;
	while (x < SIDE)
	{
		y = 0;
		while (y < SIDE)
		{
			if (y == 0 || x == 0)
				grid[x][y] = 1;
			else
				grid[x][y] = grid[x - 1][y] + grid[x][y - 1];
			printf("%ld\t", grid[x][y]);
			y++;
		}
		printf("\n");
		x++;
	}
	printf("Possible route for %dx%d: %ld\n", SIDE - 1, SIDE - 1, grid[SIDE - 1][SIDE - 1]);
	return (0);
}
