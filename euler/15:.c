/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   15:.c                                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/24 20:28:50 by adelille          #+#    #+#             */
/*   Updated: 2021/09/24 20:36:46 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

// https://en.wikipedia.org/wiki/Central_binomial_coefficient
// https://www.youtube.com/watch?v=gMlf1ELvRzc

int	main(void)
{

	long	grid[20][20];
	int		x;
	int		y;

	x = 0;
	while (x < 20)
	{
		y = 0;
		while (y < 20)
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
	printf("Possible route for 20x20: %ld\n", grid[19][19]);
	return (0);
}
