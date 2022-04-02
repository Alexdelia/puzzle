/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   32:Pandigital_products.c                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/31 13:28:32 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:51:12 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define LIMIT	2000

#define TRUE	1
#define FALSE	0

static int	ft_in(int *tab, int size, int n)
{
	int	i;

	i = 0;
	while (i < size)
	{
		if (tab[i] == n)
			return (TRUE);
		i++;
	}
	return (FALSE);
}

static int	ft_sum(int *tab, int size)
{
	int	sum;
	int	i;

	sum = 0;
	i = 0;
	while (i < size)
	{
		sum += tab[i];
		i++;
	}
	return (sum);
}

static int	ft_is_pandigital(int x, int y)
{
	int	s[10] = { 0 };
	int	n;
	int	i;

	n = x * y;
	while (x > 0)
	{
		s[x % 10] += 1;
		x /= 10;
	}
	while (y > 0)
	{
		s[y % 10] += 1;
		y /= 10;
	}
	while (n > 0)
	{
		s[n % 10] += 1;
		n /= 10;
	}
	if (s[0] != 0)
		return (FALSE);
	i = 1;
	while (i < 10)
	{
		if (s[i] != 1)
			return (FALSE);
		i++;
	}
	return (TRUE);
}

int	main(void)
{
	int		sum[100] = { 0 };
	int		i;
	int		x;
	int		y;

	i = 0;
	x = 1;
	while (x < 50)
	{
		y = 100;
		while (y < LIMIT)
		{
			if (ft_is_pandigital(x, y) == TRUE)
			{
				if (ft_in(sum, i, x * y) == FALSE)
				{
					sum[i] = x * y;
					printf("%d * %d = %d\n", x, y, sum[i]);
					i++;
				}
			}
			y++;
		}
		x++;
	}
	printf("Sum: %d\n", ft_sum(sum, i));
	return (0);
}
