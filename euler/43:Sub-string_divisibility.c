/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   43:Sub-string_divisibility.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2021/12/07 13:06:25 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdbool.h>

#include "../libft/includes/libft.h"

static bool	ssd(int x)
{	
	char	*s;
	int		i;
	int		tmp;
	
	s = ft_itoa(x);
	if (!s)
		return (false);
	i = 1;
	while (i <= 7)
	{
		tmp = s[i] - '0' + s[i + 1] - '0' + s[i + 2] - '0';
	}
	free(s);
	return (true); //
}

static bool	ft_is_pandigital(int x)
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
		return (false);
	i = 1;
	while (i <= size)
	{
		if (s[i] != 1)
			return (false);
		i++;
	}
	return (true);
}

int	main(void)
{
	int		n;
	long	sum;

	n = 123456789;
	sum = 0;
	while (n < 987654321)
	{
		if (ft_is_pandigital(n) && ssd(n))
			sum += n;
		n++;
	}
	printf("\nSum ssd pandigital: %ld\n", sum);
	return (0);
}
