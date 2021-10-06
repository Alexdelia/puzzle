/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   19:Counting_Sundays.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/06 17:26:14 by adelille          #+#    #+#             */
/*   Updated: 2021/10/06 17:52:56 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

static int	ft_month(int m, int y)
{
	m++;
	if (m == 4 || m == 6 || m == 9 || m == 11)
		return (30);
	else if (m == 2)
	{
		if ((y % 4 == 0 && y % 100 != 0) || (y % 400 == 0))
			return (29);
		return (28);
	}
	return (31);
}

int	main(void)
{
	int	d;
	int	m;
	int	y;
	int	total;

	d = 0;
	y = 1900;
	total = 0;
	while (y <= 2000)
	{
		m = 0;
		while (m < 12)
		{
			if (d == 6 && y >= 1901)
				total++;
			d = (d + ft_month(m, y)) % 7;
			m++;
		}
		y++;
	}
	printf("total sunday on first of the month: %d\n", total);
	return (0);
}
