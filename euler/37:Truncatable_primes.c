/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   37:Truncatable_primes.c                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/20 18:17:04 by adelille          #+#    #+#             */
/*   Updated: 2021/11/02 12:57:58 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <math.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>

#define	TRUE	1
#define	FALSE	0

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

static long ft_is_prime(long nb)
{
	int	div;
	int	sqr;

	div = 3;
	sqr = sqrt(nb);
	if (nb == 2)
		return (TRUE);
	if (nb <= 1 || nb % 2 == 0)
		return (FALSE);
	while (div <= sqr)
	{
		if (nb % div == 0)
			return (FALSE);
		div += 2;
	}
	return (TRUE);
}

static long	ft_next_prime(long prime)
{
	// I suppose that input will be a prime number
	prime += 2;
	while (ft_is_prime(prime) == FALSE)
		prime += 2;
	return (prime);
}

static int	ft_is_truncatable(long p)
{
	// might have int overflow
	char	*str;
	int		i;

	str = ft_itoa(p);
	i = 0;
	while (str[i])
	{
		if (ft_is_prime(atoi(&str[i])) == FALSE)
		{
			free(str);
			return (FALSE);
		}
		i++;
	}
	i = strlen(str);
	while (i > 0)
	{
		if (ft_is_prime(atoi(str)) == FALSE)
		{
			free(str);
			return (FALSE);
		}
		i--;
		str[i] = '\0';
	}
	free(str);
	return (TRUE);
}

int main(void)
{
	int		sum;
	int		i;
	long	p;
	
	p = 11;
	sum = 0;
	i = 0;
	while (i < 11 || p > INT_MAX)
	{
		if (ft_is_truncatable(p) == TRUE)
		{
			printf("%ld\n", p);
			sum += p;
			i++;
		}
		p = ft_next_prime(p);
	}
	printf("Sum of truncatable primes: %d\n", sum);
	return (0);
}
