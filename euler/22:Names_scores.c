/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   22:Names_scores.c                                  :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/06 16:45:05 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:41:18 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// calculate numbers of names and len of biggest names
#define NONAMES	5164
#define LENAMES	13

/*int	main(int ac, char **av)
{
	FILE	*fd;
	char	c;
	int		numbers_of_names;
	int		name_len;
	int		max;

	if (ac != 2)
		return (fprintf(stdout, "no input file\n") * 0 + 1);
	fd = fopen(av[1], "r");
	if (fd < 0)
		return (fprintf(stdout, "Error: Cant't open %s\n", av[1]) * 0 + 1);
	numbers_of_names = 1;
	name_len = -2;
	max = 0;
	while ((c = fgetc(fd)) != EOF)
	{
		if (c == ',')
		{
			numbers_of_names++;
			if (max < name_len)
				max = name_len;
			name_len = -2;
		}
		name_len++;
	}
	printf("Numbers of names: %d\nMax len name: %d\n", numbers_of_names, max);
	fclose(fd);
	return (0);
}*/

int	ft_score(char name[LENAMES], int place)
{
	int	i;
	int	sum;
	
	i = 0;
	sum = 0;
	while (name[i])
	{
		sum = sum + (name[i] - 'A' + 1);
		i++;
	}
	return (sum * place);
}

int	ft_comp(void const *a, void const *b)
{
	char const	*aa = (char const *)a;
	char const	*bb = (char const *)b;

	return (strcmp(aa, bb));
}

int	main(int ac, char **av)
{
	FILE	*fd;
	char	names[NONAMES][LENAMES];
	char	c;
	int		i;
	int		index;
	long	sum;

	if (ac != 2)
		return (fprintf(stdout, "no input file\n") * 0 + 1);
	fd = fopen(av[1], "r");
	if (fd < 0)
		return (fprintf(stdout, "Error: Cant't open %s\n", av[1]) * 0 + 1);
	i = 0;
	index = 0;
	while ((c = fgetc(fd)) != EOF)
	{
		if (c == ',')
		{
			names[index][i] = '\0';
			i = 0;
			index++;
		}
		else if (c != '"')
		{
			names[index][i] = c;
			i++;
		}
	}
	names[index][i] = '\0';
	index++;
	qsort(names, index, sizeof(*names), ft_comp);
	i = 0;
	sum = 0;
	while (i < NONAMES)
	{
		sum += ft_score(names[i], i + 1);
		i++;
	}
	printf("Sum of names: %ld\n", sum);
	fclose(fd);
	return (0);
}
