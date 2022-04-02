/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   45:Triangular,_pentagonal,_and_hexagonal.          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/02/08 16:10:43 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:55:54 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

//#include "../libft/inc/libft.h"
#include <stdio.h>

#define SIZE	100000

int	main(void)
{
	size_t	t[SIZE] = { 0 };
	size_t	p[SIZE] = { 0 };
	size_t	h[SIZE] = { 0 };
	size_t	ti;
	size_t	pi;
	size_t	hi;

	ti = 285;
	t[ti] = ti * (ti + 1) / 2;
	pi = 165;
	p[pi] = pi * (3 * pi - 1) / 2;
	hi = 143;
	h[hi] = hi * (2 * hi - 1);

	printf("T[%ld] = %ld\nP[%ld] = %ld\nH[%ld] = %ld\n", ti, t[ti], pi, p[pi], hi, h[hi]);

	while (ti < SIZE)
	{
		ti++;
		t[ti] = ti * (ti + 1) / 2;
		while (pi < SIZE && t[ti] > p[pi])
		{
			pi++;
			p[pi] = pi * (3 * pi - 1) / 2;
		}
		if (t[ti] == p[pi])
		{
			while (hi < SIZE && p[pi] > h[hi])
			{
				hi++;
				h[hi] = hi * (2 * hi - 1);
			}
		}
		if (t[ti] == p[pi] && t[ti] == h[hi])
			break ;
	}
	printf("T[%ld] = P[%ld] = H[%ld] = %ld\n", ti, pi, hi, t[ti]);
	return (0);
}
