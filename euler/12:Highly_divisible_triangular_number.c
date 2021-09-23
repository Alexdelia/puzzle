/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   12:Highly_divisible_triangular_number.c            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/23 19:09:37 by adelille          #+#    #+#             */
/*   Updated: 2021/09/23 20:08:40 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <unistd.h>
#include <limits.h>

static int	ft_ps(char *str)
{
	int	i;

	i = 0;
	while (str[i])
		i++;
	write(1, str, i);
	return (i);
}

static long	ft_abs(long nbr)
{
	if (nbr < 0)
		return (-nbr);
	return (nbr);
}

static void	ft_pn(long n)
{
	char	str[65] = {'0'};
	int		is_neg;
	int		len;

	is_neg = (n < 0);
	if (n == 0)
		str[0] = '0';
	len = 0;
	while (n != 0)
	{
		str[len++] = '0' + ft_abs(n % 10);
		n = (n / 10);
	}
	if (is_neg)
		str[len] = '-';
	else if (len > 0)
		len--;
	while (len >= 0)
		write(1, &str[len--], 1);
}

int	main(void)
{
	long	integer;
	long	triangular;
	long	div;
	int		n_div;
	int		percent;

	n_div = 0;
	integer = 1;
	triangular = 1;
	percent = 0;
	while (n_div <= 500)
	{
		integer++;
		triangular += integer;
		if (triangular % 2 == 0)
		{
			n_div = 1;
			div = 2;
			// brute force
			while (div * div < triangular)
			{
				if (triangular % div == 0)
					n_div++;
				div++;
				/*if (div > 2000000)
				{
					ft_ps("\r[");
					ft_pn(div * 100 / triangular);
					ft_ps("%]");
				}*/
			}
			n_div *= 2;
			/*if (div > 2000000)
			{
				ft_pn(triangular);
				ft_ps("\t");
			}*/
			if (n_div > percent)
			{
				percent = n_div;
				// printf("\r[%d%%] -> %ld", percent * 100 / 500, triangular);
				/*if (div > 2000000)
					ft_ps("\n");*/
				//ft_ps("\t  ");
				ft_pn(percent);
				ft_ps("\t<- ");
				ft_pn(triangular);
				ft_ps("\n");
			}
		}
	}
	printf("\nTriangular = %ld\t(n_div = %d | integer = %ld)\n",
			triangular, n_div, integer);
	return (0);
}
