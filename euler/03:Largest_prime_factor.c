/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   03:Largest_prime_factor.c                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/20 18:17:04 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:52:15 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <math.h>

#define NUM		600851475143
#define SQRT	775146
#define TRUE	1
#define FALSE	0

//ft_next_prime

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
	long		prime;
	long		biggest_prime;

	prime = 1;
	biggest_prime = 2;
	while (prime * prime <= NUM)
	{
		prime = ft_next_prime(prime);
		if (NUM % prime == 0)
		{
			biggest_prime = prime;
			//printf("\r[%ld%%] -> %ld", biggest_prime * 100 / SQRT, biggest_prime);
		}
	}
	printf("\nbiggest prime: %ld\n", biggest_prime);
	return (0);
}
