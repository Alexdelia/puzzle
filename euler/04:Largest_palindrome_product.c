/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Largest_palindrome_product.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/20 18:59:22 by adelille          #+#    #+#             */
/*   Updated: 2021/09/20 20:22:45 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#define	TRUE	1
#define FALSE	0

static int	ft_is_palindrome(int p)
{
	// only taking 123321 format
	char	n[6];
	int		i;

	i = 5;
	while (i >= 0)
	{
		n[i] = p % 10 + '0';
		p /= 10;
		i--;
	}

	if (n[0] != n[5]
			|| n[1] != n[4]
			|| n[2] != n[3])
		return (FALSE);
	return (TRUE);
}

int	main(void)
{
	int	palindrome;
	int	x;
	int	y;

	palindrome = 0;
	x = 999;
	while (x > 850)
	{
		y = 999;
		while (y > 850)
		{
			if (ft_is_palindrome(x * y) == TRUE)
			{
				if (palindrome < x * y)
					palindrome = x * y;
				break ;
			}
			y--;
		}
		x--;
	}
	printf("largest palindrome: %d\n", palindrome);
	return (0);
}
