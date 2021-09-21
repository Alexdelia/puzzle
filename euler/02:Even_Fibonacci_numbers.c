/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Even_Fibonacci_numbers.c                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/20 18:02:50 by adelille          #+#    #+#             */
/*   Updated: 2021/09/20 18:13:14 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

int	main(void)
{
	long	total;
	int		fib1;
	int		fib2;

	total = 2;
	fib1 = 1;
	fib2 = 2;
	while (fib2 < 4000000)
	{
		fib1 += fib2;
		if (fib1 % 2 == 0)
			total += fib1;
		if (fib1 >= 4000000)
			break ;
		fib2 += fib1;
		if (fib2 % 2 == 0)
			total += fib2;
	}

	printf("total: %ld\n", total);
	return (0);
}
