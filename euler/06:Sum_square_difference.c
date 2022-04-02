/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   06:Sum_square_difference.c                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/21 20:29:54 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:52:29 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

int	main(void)
{
	int	i;
	long	t_sumsqr;
	long	t_sqrsum;

	i = 1;
	t_sumsqr = 0;
	t_sqrsum = 0;
	while (i <= 100)
	{
		t_sumsqr += i * i;
		t_sqrsum += i;
		i++;
	}
	t_sqrsum *= t_sqrsum;
	printf("%ld - %ld = %ld\n", t_sqrsum, t_sumsqr, t_sqrsum - t_sumsqr);
	return (0);
}
