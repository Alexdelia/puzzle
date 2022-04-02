/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   10:Summation_of_primes.c                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/21 20:36:09 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:40:31 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <math.h>

#define TRUE	1
#define FALSE	0

static long	ft_is_prime(long nb)
{
	// I suppose that I only have odd numbers in input
	int	div;
	int	sqr;

	div = 3;
	sqr = sqrt(nb);
	while (div <= sqr)
	{
		if (nb % div == 0)
			return (FALSE);
		div += 2;
	}
	return (TRUE);
}

static long	ft_next_prime(long prime)
{
	// I suppose that input will be a prime number
	prime += 2;
	while (ft_is_prime(prime) == FALSE)
		prime += 2;
	return (prime);
}

int	main(void)
{
	long	prime;
	long	sum;
	int		percent;

	prime = 1;
	sum = -1;
	percent = 0;
	while (prime < 2000000)
	{
		prime = ft_next_prime(prime);
		sum += prime;
		if (prime * 100 / 2000000 > percent)
		{
			percent = prime * 100 / 2000000;
			printf("\r[%d%%] -> %ld", percent, prime);
		}
	}
	printf("\r[100%%] Sum: %ld\n", sum);
	return (0);
}
