/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   52:Permuted_multiples.c                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2022/03/30 13:30:52 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../libft/inc/libft.h"

#include <stdio.h>

#define MAX	1000000

static void	pm_bzero(size_t *a)
{
	size_t	i;

	i = -1;
	while (++i < 10)
		a[i] = 0;
}

static void	fill(size_t *dst, size_t n)
{
	while (n > 0)
	{
		dst[n % 10] += 1;
		n /= 10;
	}
}

static bool	pm_comp(const size_t *base, const size_t *comp)
{
	size_t	i;

	i = 0;
	while (i < 10)
	{
		if (base[i] != comp[i])
			return (false);
		i++;
	}
	return (true);
}

static bool	is_permuted_multiples(const size_t n)
{
	size_t	base[10] = { 0 };
	size_t	comp[10] = { 0 };
	size_t	i;

	fill(base, n);

	i = 2;
	while (i <= 6)
	{
		pm_bzero(comp);
		fill(comp, n * i);
		if (!pm_comp(base, comp))
			return (false);
		i++;
	}
	return (true);
}

int	main(void)
{
	size_t	n;

	n = 2;
	while (n < MAX)
	{
		if (is_permuted_multiples(n))
			break ;
		n++;
	}
	printf("smalest permuted multiples:\n%ld\n", n);
	return (0);
}
