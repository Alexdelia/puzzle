/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   09:Special_Pythagorean_triplet.c                   :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/22 20:43:27 by adelille          #+#    #+#             */
/*   Updated: 2021/09/22 21:11:21 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

int	ft_sum(int m, int n)
{
	return (((m * m) - (n * n))
			+ (2 * m * n)
			+ ((m * m) + (n * n)));
}

int	main(void)
{
	int	m;
	int	n;

	m = 1;
	while (m < 50)
	{
		n = 1;
		while (n < 50)
		{
			if ((m * m) - (n * n) > 0 && ft_sum(m, n) == 1000)
			{
				printf("m = %d | n = %d\na = %d | b = %d | c = %d\n",
						m, n, (m * m) - (n * n), 2 * m * n, (m * m) + (n * n));
				return (0);
			}
			n++;
		}
		m++;
	}
	printf("No\n");
	return (0);
}
