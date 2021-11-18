/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   42:Coded_triangle_numbers.c                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/06 16:45:05 by adelille          #+#    #+#             */
/*   Updated: 2021/11/19 00:01:47 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

bool	ft_is_triangle(char *word)
{
	int	i;
	int	n;
	//int	t;
	
	i = 0;
	n = 0;
	while (word[i])
	{
		n += word[i] - 'A' + 1;
		i++;
	}
	//printf("%s|%d\n", word, n);
	if (strcmp(word, "SKY") == 0)
		printf("-- %s\t| %d --\n", word, n);
	/*i = 1;
	t = 0;
	while (t < n)
	{
		t = i / 2 * (i + 1);
		i++;
	}
	if (t == n)
	{
		printf("%s\t| %d\n", word, n);
		return (true);
	}*/
	if (sqrt(8 * n + 1) == (int)(sqrt(8 * n + 1)))
		return (true);
	return (false);
}

int	main(int ac, char **av)
{
	FILE	*fd;
	char	word[50];
	int		sum;

	if (ac != 2)
		return (fprintf(stderr, "no input file\n") * 0 + 1);
	fd = fopen(av[1], "r");
	if (fd < 0)
		return (fprintf(stderr, "Error: Cant't open %s\n", av[1]) * 0 + 1);
	sum = 0;
	fscanf(fd, "\"%[A-Z]\"", word);
	//printf("%s\t", word);
	if (ft_is_triangle(word) == true)
		sum++;
	while (fscanf(fd, ",\"%[A-Z]\"", word) != EOF)
	{
		if (ft_is_triangle(word) == true)
			sum++;
		//printf("%s\t", word);
	}
	printf("Sum of coded triangle numbers: %d\n", sum);
	fclose(fd);
	return (0);
}
