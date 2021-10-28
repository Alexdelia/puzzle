/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   30:Digit_fifth_powers.c                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/28 17:30:19 by adelille          #+#    #+#             */
/*   Updated: 2021/10/28 18:01:27 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdlib.h>
#include <math.h>

#define TRUE	1
#define FALSE	0

#define LIMIT	1000000

static int	ft_is_true(int n, int e)
{
	int	sum;
	int	d;

	sum = 0;
	d = n;
	while (d > 0 && sum <= n)
	{
		sum += (int)pow(d % 10, e);
		d /= 10;
	}
	if (sum == n)
		return (TRUE);
	return (FALSE);
}

int	main(int ac, char **av)
{
	int		e;
	int		i;
	int		res;
	int		last_res;
	int		percent;
	long	sum;

	if (ac == 2)
		e = atoi(av[1]);
	else
		e = 5;
	i = 2;
	sum = 0;
	percent = 0;
	while (i < LIMIT)
	{
		res = ft_is_true(i, e);
		if (res == TRUE)
		{
			last_res = i;
			sum += i;
		}
		if (i / LIMIT * 100 > percent)
		{
			percent = 1 / LIMIT * 100;
			printf("\rProcess [%d%%]\t%d", percent, last_res);
		}
		i++;
	}
	printf("Sum: %ld\n", sum);
	return (0);
}
