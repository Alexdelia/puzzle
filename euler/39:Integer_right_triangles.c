/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   39:Integer_right_triangles.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/07 17:30:52 by adelille          #+#    #+#             */
/*   Updated: 2021/11/07 17:39:09 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

int	main(void)
{
	long	res;
	long	solutions;
	long	p;
	long	a;
	int		n_sol;

	solutions = -1;
	res = -1;
	p = 2;
	while (p <= 1000)
	{
		n_sol = 0;
		a = 2;
		while (a < p / 3)
		{
			if (p * (p - 2 * a) % (2 * (p - a)) == 0)
				n_sol++;
			a++;
		}
		if (n_sol > solutions)
		{
			solutions = n_sol;
			res = p;
		}
		p += 2;
	}
	printf("res: %ld\nsolutions: %ld\n", res, solutions);
	return (0);
}
