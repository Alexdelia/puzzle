/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   27:Quadratic_primes.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/28 18:20:54 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:47:27 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define TRUE	1
#define FALSE	0

static int	ft_is_prime(int nb)
{
	int	div;

	if (nb % 2 == 0 || nb <= 2)
		return (FALSE);
	div = 3;
	while (div * div <= nb)
	{
		if (nb % div == 0)
			return (FALSE);
		div += 2;
	}
	return (TRUE);
}

static int	ft_next_prime(int prime)
{
	// I suppose that input will be a prime number
	prime += 2;
	while (ft_is_prime(prime) == FALSE)
		prime += 2;
	return (prime);
}

static int	ft_quadratic_len(int a, int b)
{
	int	i;

	i = 0;
	while (ft_is_prime(i * i + a * i + b))
		i++;
	return (i);
}

int	main(void)
{
	int	a;
	int	b;
	int	max_a;
	int	max_b;
	int	size;
	int	max_size;

	max_size = -1;
	a = -999;
	while (a < 1000)
	{
		b = 3;
		while (b <= 1000)
		{
			size = ft_quadratic_len(a, b);
			if (size > max_size)
			{
				max_size = size;
				max_a = a;
				max_b = b;
			}
			b = ft_next_prime(b);
		}
		a += 2;
	}
	printf("%d * %d = %d\twith %d numbers of primes\n",
		max_a, max_b, max_b * max_a, max_size);
	return (0);
}
