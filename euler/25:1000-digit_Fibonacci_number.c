/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   25:1000-digit_Fibonacci_number.c                   :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/25 15:43:43 by adelille          #+#    #+#             */
/*   Updated: 2021/10/25 16:02:09 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

int	ft_fib(long double prev, long double current, int index, long double limit)
{
	if (current > limit)
		return (index);
	return (ft_fib(current, prev + current, index + 1, limit));
}

int	main(void)
{
	long double	limit;
	int			i;

	i = 1;
	limit = 1;
	while (i < 1000 || limit < 0)
	{
		limit *= 10;
		i++;
	}
	if (limit < 0)
		printf("fuck\n");
	else
		printf("index first 1k digit number: %d\n", ft_fib(1, 1, 2, limit));
	return (0);
}
