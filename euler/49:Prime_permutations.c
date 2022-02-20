/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   49:Prime_permutations.c                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/20 17:22:04 by adelille          #+#    #+#             */
/*   Updated: 2022/02/20 17:30:49 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../libft/inc/libft.h"
#include <stdio.h>

bool	permute(size_t n1, size_t n2, size_t n3)
{
	size_t	s1[10] = { 0 };
	size_t	s2[10] = { 0 };
	size_t	s3[10] = { 0 };
	size_t	i;

	while (n1)
	{
		s1[n1 % 10] += 1;
		n1 = n1 / 10;
	}
	while (n2)
	{
		s2[n2 % 10] += 1;
		n2 = n2 / 10;
	}
	while (n3)
	{
		s3[n3 % 10] += 1;
		n3 = n3 / 10;
	}

	i = 0;
	while (i < 10)
	{
		if (s1[i] != s2[i] || s1[i] != s3[i])
			return (false);
		i++;
	}
	return (true);
}

int	main(void)
{
	size_t	i;

	i = 1000;
	while (i < 10000 - (3330 * 2))
	{
		if (ft_is_prime_ul(i) && ft_is_prime_ul(i + 3330)
				&& ft_is_prime_ul(i + (3330 * 2))
				&& permute(i, i + 3330, i + (3330 * 2)))
			printf("%ld-%ld-%ld\n", i, i + 3330, i + (3330 * 2));
		i++;
	}
	return (0);
}
