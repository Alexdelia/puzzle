/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Smallest_multiple.c                                :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/21 20:14:32 by adelille          #+#    #+#             */
/*   Updated: 2021/09/21 20:21:35 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

int	main(void)
{
	long	n;
	int		div;
	int		b;

	n = 0;
	b = 0;
	while (b == 0)
	{
		n += 20;
		div = 19;
		while (div > 2)
		{
			if (n % div != 0)
				break ;
			div--;
		}
		if (div == 2)
			b++;
	}
	printf("Smallest multiple: %ld\n", n);
	return (0);
}
