/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   21:Amicable_numbers.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/10 14:53:54 by adelille          #+#    #+#             */
/*   Updated: 2021/10/10 15:38:59 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define TRUE	1
#define FALSE	0

#define	LIMIT	10000

void	ft_init_dict(int d[LIMIT], int amicable[LIMIT])
{
	int	i;

	i = 0;
	while (i < LIMIT)
	{
		d[i] = -1;
		amicable[i] = FALSE;
		i++;
	}
}

long	ft_is_amicable(int d[LIMIT], int amicable[LIMIT], int n, int from)
{
	int	i;
	int	sum;

	if (n >= LIMIT || n == from)
		return (FALSE);
	if (d[n] == -1)
	{
		i = 1;
		sum = 0;
		while (i < n)
		{
			if (n % i == 0)
				sum += i;
			i++;
		}
		if (n < LIMIT)
			d[n] = sum;
	}
	if (from == -1)
	{
		if (ft_is_amicable(d, amicable, d[n], n) == TRUE)
		{
			amicable[n] = TRUE;
			amicable[d[n]] = TRUE;
			return (n + d[n]);
		}
		return (0);
	}
	else if (d[n] == from)
		return (TRUE);
	return (FALSE);
}

int	main(void)
{
	int		d[LIMIT];
	int		amicable[LIMIT];
	int		i;
	long	sum;

	ft_init_dict(d, amicable);
	i = 2;
	sum = 0;
	while (i < LIMIT)
	{
		if (amicable[i] == FALSE)
			sum += ft_is_amicable(d, amicable, i, -1);
		i++;
	}
	printf("Sum of Amicalbe numbers under %d = %ld\n", LIMIT, sum);
}
