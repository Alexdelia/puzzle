/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   31:Coin_sums.c                                     :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/30 17:52:52 by adelille          #+#    #+#             */
/*   Updated: 2021/10/30 18:05:03 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define MAX	200

int	main(void)
{
	int	coins[8] = { 1, 2, 5, 10, 20, 50, 100, 200 };
	int	solutions[MAX + 1] = { 0 };
	int	x;
	int	y;

	solutions[0] = 1;
	x = 0;
	while (x < 8)
	{
		y = coins[x];
		while (y <= MAX)
		{
			solutions[y] += solutions[y - coins[x]];
			y++;
		}
		x++;
	}
	/*x = 0;
	while (x <= MAX)
	{
		printf("%d\t", solutions[x]);
		x++;
	}*/
	printf("%d ways to make 2 pound\n", solutions[MAX]);
	return (0);
}
