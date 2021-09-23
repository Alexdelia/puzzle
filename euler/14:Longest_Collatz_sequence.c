/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   14:Longest_Collatz_sequence.c                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/23 20:59:00 by adelille          #+#    #+#             */
/*   Updated: 2021/09/23 21:23:36 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

static long	ft_collatz(long n)
{
	long	iteration;

	iteration = 1;
	while (n > 1)
	{
		if (n % 2 == 0)
			n /= 2;
		else
			n = n * 3 + 1;
		iteration++;
	}
	return (iteration);
}

int	main(void)
{
	int		integer;
	int		b_int;
	long	iteration;
	long	b_ite;

	integer = 3;
	b_ite = 0;
	while (integer < 1000000)
	{
		iteration = ft_collatz(integer);
		if (integer == 13)
			printf("13 has chain of %ld\n", iteration);
		if (iteration > b_ite)
		{
			b_ite = iteration;
			b_int = integer;
		}
		integer += 2;
	}
	printf("Start: %d (chain of: %ld)\n", b_int, b_ite);
	return (0);
}
