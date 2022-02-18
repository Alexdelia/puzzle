/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   46:Goldbach's_other_conjecture.c                   :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/08 16:10:43 by adelille          #+#    #+#             */
/*   Updated: 2022/02/18 17:51:41 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../libft/inc/libft.h"
#include <stdio.h>

#define LIMIT	10000

int	main(void)
{
	size_t	odd;
	size_t	prime;
	size_t	square;

	odd = 7;
	prime = 1;

	while (prime < odd)
	{
		odd += 2;
		while (ft_is_prime_ul(odd))
			odd += 2;
		prime = 1;
		while (prime < odd && odd != (prime + (2 * (square * square))))
		{
			prime = ft_next_prime_ul(prime);
			square = 1;
			while (square < LIMIT && odd > (prime + (2 * (square * square))))
				square++;
		}
		if (prime > odd)
			printf("\n%ld\n", odd);
		else
			printf("%ld = %ld + 2x%ld²\n", odd, prime, square);
	}
	return (0);
}
