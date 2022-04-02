/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   26:Reciprocal_cycles.c                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/25 16:13:24 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:44:09 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

static int	ft_rc_len(int d)
{
	int	seen[1000] = { -1 };
	int	n;
	int	index;

	n = 1;
	index = 0;
	while (seen[n] == 0 && n != 0)
	{
		seen[n] = index;
		n *= 10;
		n %= d;
		index++;
	}
	return (index - seen[n] + 1);
}

int	main(void)
{
	int		d;
	int		tmp;
	int		big;
	int		index;

	d = 2;
	big = -1;
	while (d < 1000)
	{
		tmp = ft_rc_len(d);
		if (tmp > big)
		{
			big = tmp;
			index = d;
		}
		d++;
	}
	printf("Longest reciprocal cycles is 1/%d, for length of %d\n", d, big);
	return (0);
}
