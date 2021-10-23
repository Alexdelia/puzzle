/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   A71.c                                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/23 12:37:47 by adelille          #+#    #+#             */
/*   Updated: 2021/10/23 12:48:48 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "stdio.h"
#include "string.h"

int	main(void)
{
	unsigned short	n;
	unsigned short	size;
	char			w[101];

	scanf("%hu", &n);
	while (n > 0)
	{
		scanf("\n%s", w);
		size = (unsigned short)strlen(w);
		if (size > 10)
			printf("%c%d%c\n", w[0], size - 2, w[size - 1]);
		else
			printf("%s\n", w);
		n--;
	}
	return (0);
}
