/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   23:Non-abundant_sums.c                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/22 18:21:00 by adelille          #+#    #+#             */
/*   Updated: 2021/10/22 19:43:06 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdbool.h>

#define	MAX	28123

static int	ft_sum_divisor(int n)
{
	int	i;
	int	sum;

	i = 2;
	sum = 1;
	while (i * 2 <= n)
	{
		if (n % i == 0)
			sum += i;
		i++;
	}
	return (sum);
}

static int	ft_find_all_ab(int ab[10000])
{
	int	n;
	int	sum;
	int	i;

	i = 0;
	n = 12;
	while (n <= MAX)
	{
		sum = ft_sum_divisor(n);
		if (sum > n)
		{
			ab[i] = n;
			i++;
		}
		n++;
	}
	return (i);
}

static void	ft_add_all_ab(int ab[10000], int size, bool b[MAX + 1])
{
	int	i;
	int	x;

	i = 0;
	while (i < size)
	{
		x = i;
		while (x < size && ab[i] + ab[x] <= MAX)
		{
			b[ab[i] + ab[x]] = true;
			x++;
		}
		i++;
	}
}

static int	ft_add_all_false(bool b[MAX + 1])
{
	int	sum;
	int	i;

	sum = 0;
	i = 0;
	while (i <= MAX)
	{
		if (b[i] == false)
			sum += i;
		i++;
	}
	return (sum);
}

int	main(void)
{
	int		ab[10000] = { -1 };// don't do this at home
	int		size;
	bool	b[MAX + 1] = { false };

	// find all ab <= MAX
	size = ft_find_all_ab(ab);
	// add them together one by one, turn bool array to TRUE for each result
	ft_add_all_ab(ab, size, b);
	// add all number in array that are FALSE
	printf("Non-abundant sums: %d\n", ft_add_all_false(b));

	return (0);
}
