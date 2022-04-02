/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   67:Maximum_path_sum_II.c                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/10/06 16:45:05 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:44:53 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

#define ROW	100

int	row[ROW] = {0};
int	last_row[ROW] = {0};

void	ft_process_row(int irow)
{
	int	i;
	int	l;
	int	r;

	i = 0;
	while (i <= irow)
	{
		if (i == 0)
			row[i] += last_row[i];
		else if (i == irow)
			row[i] += last_row[i - 1];
		else
		{
			l = row[i] + last_row[i - 1];
			r = row[i] + last_row[i];
			row[i] = (l > r ? l : r); 
		}
		i++;
	}
	i = 0;
	while (i <= irow)
	{
		last_row[i] = row[i];
		i++;
	}
}

int	ft_max_in_row(int size)
{
	int	max;
	int	i;

	max = -1;
	i = 0;
	while (i < size)
	{
		if (last_row[i] > max)
			max = last_row[i];
		i++;
	}
	return (max);
}

int	main(int ac, char **av)
{
	FILE	*fd;
	char	c;
	int		tmp;
	int		irow;

	if (ac != 2)
		return (fprintf(stdout, "no input file\n") * 0 + 1);
	fd = fopen(av[1], "r");
	if (fd < 0)
		return (fprintf(stdout, "Error: Cant't open %s\n", av[1]) * 0 + 1);
	tmp = 0;
	irow = 0;
	while ((c = fgetc(fd)) != EOF)
	{
		if (c == ' ')
		{
			row[irow] = tmp;
			tmp = 0;
			irow++;
		}
		else if (c == '\n')
		{
			row[irow] = tmp;
			ft_process_row(irow);
			tmp = 0;
			irow = 0;
		}
		else
		{
			tmp *= 10;
			tmp += (c - '0');
		}
	}
	row[irow] = tmp;
	ft_process_row(irow);
	printf("Maximum path sum: %d\n", ft_max_in_row(ROW));
	fclose(fd);
	return (0);
}
