/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   34:Digit_factorials.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/30 09:40:48 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:38:56 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define LIMIT	100000

#define TRUE	1
#define FALSE	0

long	ft_factorial(int n)
{
	long	f;

	f = 1;
	while (n > 1)
	{
		f *= n;
		n--;
	}
	return (f);
}

int	ft_digit_factorials(int n)
{
	long	sum;
	int		d;

	sum = 0;
	d = n;
	while (d > 0 && sum <= n)
	{
		sum += ft_factorial(d % 10);
		d /= 10;
	}
	if (sum == n)
		return (TRUE);
	return (FALSE);
}

int	main(void)
{
	long	sum;
	int		i;

	sum = 0;
	i = 10;
	while (i < LIMIT)
	{
		if (ft_digit_factorials(i) == TRUE)
			sum += i;
		i++;
	}
	printf("sum: %ld\n", sum);
	return (0);
}
