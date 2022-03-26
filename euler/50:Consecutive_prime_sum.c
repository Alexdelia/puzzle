/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   50:Consecutive_prime_sum.c                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/20 17:22:04 by adelille          #+#    #+#             */
/*   Updated: 2022/03/26 18:42:49 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../libft/inc/libft.h"
#include <stdio.h>

#define MAX	1000000
#define MIN	1000

int	main(void)
{
	size_t	start;
	size_t	current;
	size_t	sum;
	size_t	size;
	size_t	big_size;

	big_size = 0;
	start = 2;
	while (start / 2 < MAX)
	{
		size = 1;
		sum = start;
		current = start;
		while (sum < MAX)
		{
			current = ft_next_prime_ul(current);
			sum += current;
			size++;
			if (size > big_size && ft_is_prime_ul(sum))
			{
				big_size = size;
				printf("\rconsecutive: %ld  sum: %ld  (%ld-%ld)",
						size, sum, start, current);
			}
		}
		start = ft_next_prime_ul(start);
	}
	printf("\n");
	return (0);
}
