/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Temperature.c                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:57:17 by adelille          #+#    #+#             */
/*   Updated: 2022/05/20 10:42:56 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Codingame Puzzle
*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

int	main(void)
{
	int	n, c, i, t;

	scanf("%d", &n);
	c = (n > 0 ? 10001 : 0);
	while (i < n)
	{
		scanf("%d", &t);
		c = abs(t) < abs(c) ? t : c;
		if (abs(t) == abs(c) && t > c)
			c = t;
		i++;
	}
	printf("%d\n", c);
	return (0);
}
