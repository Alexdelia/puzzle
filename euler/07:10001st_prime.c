/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   07:10001st_prime.c                                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/21 20:36:09 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:49:20 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <math.h>

#define TRUE	1
#define FALSE	0

static long ft_is_prime(long nb)
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

int main(void)
{
	long	prime;
	int		i;
	int		percent;

	prime = 1;
	i = 2;
	percent = 0;
	while (i <= 10001)
	{
		prime = ft_next_prime(prime);
		i++;
		if (i * 100 / 10000 > percent)
		{
			percent = i * 100 / 10000;
			printf("\r[%d%%] -> %ld", percent, prime);
		}
	}
	printf("\r[%d%%] -> %ld", percent, prime);
	printf("\n10001st prime: %ld\n", prime);
	return (0);
}
