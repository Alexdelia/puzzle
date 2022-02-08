/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   43:Sub-string_divisibility.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2022/02/08 16:04:56 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdbool.h>

#include "../libft/inc/libft.h"

#

static bool	ssd(char *str)
{	
	size_t	i;
	int		tmp;

	i = 1;
	while (i <= 8)
	{
		tmp = ((str[i] - '0') * 100) + ((str[i + 1] - '0') * 10) + str[i + 2] - '0';
		switch (i)
		{
			case (1):
				if (tmp % 2)
					return (false);
				break ;
			case (2):
				if (tmp % 3)
					return (false);
				break ;
			case (3):
				if (tmp % 5)
					return (false);
				break ;
			case (4):
				if (tmp % 7)
					return (false);
				break ;
			case (5):
				if (tmp % 11)
					return (false);
				break ;
			case (6):
				if (tmp % 13)
					return (false);
				break ;
			case (7):
				if (tmp % 17)
					return (false);
				break ;
		}
		++i;
	}
	return (true); //
}

static bool	ft_is_pandigital(long x, char *str)
{
	int		s[10] = { 0 };
	size_t	size;
	size_t	i;

	size = 0;
	i = 9;
	while (x > 0)
	{
		s[x % 10] += 1;
		if (s[x % 10] > 1)
			return (false);
		str[i] = x % 10 + '0';
		--i;
		x /= 10;
		++size;
	}
	return (true);
}

int	main(void)
{
	long	n;
	long	sum;
	char	*str;

	if (!ssd("1406357289"))
		return (1);
	str = (char *)malloc(sizeof(char) * 11);
	if (!str)
		return (2);
	str[10] = '\0';
	n = 1023456789;
	sum = 0;
	while (n <= 9876543210)
	{
		//printf("\r%ld", n);
		if (ft_is_pandigital(n, str))
		{
			printf("\r%s", str);
			if (ssd(str))
			{
				printf("\r%s\n", str);
				//printf(" true\n");
				sum += n;
			}
		}
		++n;
	}
	printf("\nSum ssd pandigital: %ld\n", sum);
	free(str);
	return (0);
}
