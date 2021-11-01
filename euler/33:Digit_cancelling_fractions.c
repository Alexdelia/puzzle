/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   33:Digit_cancelling_fractions.c                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/01 21:49:05 by adelille          #+#    #+#             */
/*   Updated: 2021/11/01 22:04:38 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

// 42 norm (norminette) doesn't allow for loop in v3

int	main(void)
{
	int	a;
	int	b;
	int	c;

	a = 1;
	while (a < 10)
	{
		b = 1;
		while (b < 10)
		{
			c = 1;
			while (c < 10 && a != b)
			{
				if (10 * b * c == 9 * a * b + a * c)
				{
					printf("%d%d / %d%d = %d / %d\n", b, c, c, a, b, a);
				}
				c++;
			}
			b++;
		}
		a++;
	}
	return (0);
}
