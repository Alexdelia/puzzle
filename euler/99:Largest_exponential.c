/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   99:Largest_exponential.c                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/30 19:40:15 by adelille          #+#    #+#             */
/*   Updated: 2021/10/30 19:57:03 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <math.h>

int	main(int ac, char **av)
{
	FILE	*fd;
	int		a;
	int		b;
	int		x;
	int		y;
	int		i;
	int		big_i;
	double	big;

	if (ac != 2)
		return (printf("No input file\n") * 0 + 1);
	fd = fopen(av[1], "r");
	if (fd < 0)
		return (printf("Error: Cant't open %s\n", av[1]) * 0 + 1);
	big = -1.0;
	i = 1;
	while (fscanf(fd, "%d,%d\n", &a, &b) != EOF)
	{
		if (b * log(a) > big)
		{
			big = b * log(a);
			big_i = i;
			//printf("i=%d\n", i);
		}
		i++;
	}
	printf("Largest exponential: %d\n", big_i);
	fclose(fd);
	return (0);
}
