/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   35:Circular_primes.c                               :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2021/11/07 17:26:32 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#define LIMIT	1000000
#define TAB		10000000

bool	ft_is_prime(int n)
{
	int	div;

	if (n == 2 || n == 3)
		return (true);
	if (n <= 1 || n % 2 == 0 || n % 3 == 0)
		return (false);
	div = 5;
	while (div * div <= n)
	{
		if (n % div == 0 || n % (div + 2) == 0)
			return (false);
		div += 6;
	}
	return (true);
}

int	ft_next_prime(int n)
{
	if (n < 2)
		return (2);
	if (n % 2 == 0)
		n++;
	else
		n += 2;
	while (ft_is_prime(n) == false)
		n += 2;
	return (n);
}

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

int	ft_rotate(char *str)
{
	int		size;
	int		i;
	char	*tmp;

	tmp = strdup(str);
	i = 1;
	size = strlen(str);
	while (i < size)
	{
		str[i] = tmp[i - 1];
		i++;
	}
	str[0] = tmp[size - 1];
	//printf("%s\t", str);
	free(tmp);
	return (0);
}

int	ft_circular_prime(int n, int p[LIMIT * 10])
{
	char	*str;
	int		total;
	int		i;
	bool	b;

	if (p[n] == 1)
		return (0);
	p[n] = 1;
	total = 1;
	str = ft_itoa(n);
	i = strlen(str);
	b = true;
	while (b == true && i > 1)
	{
		ft_rotate(str);
		n = atoi(str);
		b = ft_is_prime(n);
		if (b == true)
		{
			p[n] = 1;
			total++;
		}
		else
		{
			free(str);
			return (0);
		}
		i--;
	}
	free(str);
	return (total);
}

int	main(void)
{
	int		*p;
	int		n;
	int		i;

	p = (int *)malloc(sizeof(int) * TAB);
	if (!p)
		return (printf("Malloc error\n") * 0 + 1);
	i = 0;
	while (i < TAB)
	{
		p[i] = 0;
		i++;
	}
	i = -1;
	n = 2;
	while (n < LIMIT)
	{
		i += ft_circular_prime(n, p);
		n = ft_next_prime(n);
	}
	printf("Total circular primes: %d\n", i);
	return (0);
}
