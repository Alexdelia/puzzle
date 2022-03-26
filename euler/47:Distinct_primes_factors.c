/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   47:Distinct_primes_factors.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/20 17:22:04 by adelille          #+#    #+#             */
/*   Updated: 2022/03/26 20:01:57 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../libft/inc/libft.h"
#include <stdio.h>
#include <math.h>

#define MAX		1000000
#define MIN		647
#define FACTORS	4

size_t	n_distinct_prime(size_t n)
{
	size_t	res;
	//size_t	sqrt_n;
	size_t	prime;

	res = 0;
	//sqrt_n = sqrt(n);
	prime = 2;
	while (prime < n)
	{
		if (n % prime == 0)
			res++;
		prime = ft_next_prime_ul(prime);
	}
	return (res);
}

int	main(void)
{
	size_t	i;
	size_t	consecutive;

	i = MIN;
	while (i < MAX)
	{
		consecutive = 0;
		while (n_distinct_prime(i) >= FACTORS)
		{
			consecutive++;
			i++;
		}
		if (consecutive >= FACTORS)
		{
			consecutive++;
			while (consecutive + 1 > 0)
			{
				printf("%ld (%ld)\n", i - consecutive, n_distinct_prime(i - consecutive));
				consecutive--;
			}
			printf("\n");
			return (0);
		}
		i++;
	}
	return (0);
}
