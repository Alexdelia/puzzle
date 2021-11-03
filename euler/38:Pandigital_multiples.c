/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   38:Pandigital_multiples.c                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2021/11/03 18:16:08 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define TRUE	1
#define FALSE	0

static int	ft_is_pandigital(int x, int y)
{
	int	s[10] = { 0 };
	int	i;

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
	int	n;

	n = 9999;
	while (n >= 9000)
	{
		if (ft_is_pandigital(n, n * 2) == TRUE)
			break ;
		n--;
	}
	printf("Largest pandigital: %d%d\n", n, n * 2);
	return (0);
}
