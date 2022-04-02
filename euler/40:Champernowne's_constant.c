/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   40:Champernowne's_constant.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:18:15 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:46:00 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TRUE	1
#define FALSE	0

static int	ft_abs(int nbr)
{
	if (nbr < 0)
		nbr = -nbr;
	return (nbr);
}

static void	ft_strrev(char *str)
{
	size_t	len;
	size_t	i;
	char	tmp;

	len = strlen(str);
	i = 0;
	while (i < len / 2)
	{
		tmp = str[i];
		str[i] = str[len - i - 1];
		str[len - i - 1] = tmp;
		i++;
	}
}

static char	*ft_itoa(int n)
{
	char	*str;
	int		is_neg;
	size_t	len;

	is_neg = (n < 0);
	str = calloc(11 + is_neg, sizeof(*str));
	if (!str)
		return (NULL);
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
	ft_strrev(str);
	return (str);
}

int	main(void)
{
	char	*t;
	long	prod;
	int		d;
	int		n;
	int		i;

	prod = 1;
	d = 1;
	n = 1;
	while (n <= 1000000)
	{
		i = 0;
		t = ft_itoa(d);
		while (t[i])
		{
			if (n == 1 || n == 10 || n == 100
				|| n == 1000 || n == 10000
				|| n == 100000 || n == 1000000)
				prod *= (t[i] - '0');
			n++;
			i++;
		}
		free(t);
		d++;
	}
	printf("Product: %ld\n", prod);
	return (0);
}
