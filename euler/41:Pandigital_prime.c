/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   41:Pandigital_prime.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2021/11/04 14:40:16 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <math.h>
#include <stdbool.h>

#define TRUE	1
#define FALSE	0


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

int	ft_prev_prime(int n)
{
	if (n <= 2)
		return (2);
	if (n % 2 == 0)
		n--;
	else
		n -= 2;
	while (ft_is_prime(n) == false)
		n -= 2;
	return (n);
}

static int	ft_is_pandigital(int x)
{
	int	s[10] = { 0 };
	int	size;
	int	i;

	size = 0;
	while (x > 0)
	{
		s[x % 10] += 1;
		x /= 10;
		size++;
	}
	if (s[0] != 0)
		return (FALSE);
	i = 1;
	while (i <= size)
	{
		if (s[i] != 1)
			return (FALSE);
		i++;
	}
	return (TRUE);
}

int	main(void)
{
	int	n;
	int	i;

	i = 1;
	//n = ft_prev_prime(987654321);
	n = ft_prev_prime(87654321);
	//n = ft_prev_prime(7654321);
	printf("start: %d\n", n);
	while (ft_is_pandigital(n) == FALSE)
	{
		if (i % 300 == 0)
			printf("\r%d", n);
		i++;
		n = ft_prev_prime(n);
	}
	printf("\nBiggest pandigital prime: %d\n", n);
	return (0);
}
