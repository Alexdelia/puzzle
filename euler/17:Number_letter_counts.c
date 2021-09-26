/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   17:Number_letter_counts.c                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/26 17:34:01 by adelille          #+#    #+#             */
/*   Updated: 2021/09/26 18:54:55 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>

char	g_dict[1002][40];

static int	ft_strlen(const char *str)
{
	int	i;

	i = 0;
	while (str[i])
		i++;
	return (i);
}

static int	ft_strcpy(char *dst, const char *src)
{
	int	i;
	int	size;

	i = 0;
	size = ft_strlen(src);
	while (i < size - 1 && src[i])
	{
		dst[i] = src[i];
		i++;
	}
	dst[i] = '\0';
	return (i);
}

static int	ft_stracpy(char *dst, const char *src)
{	
	int	d;
	int	s;
	int	size;

	d = 0;
	while (dst[d])
		d++;
	size = ft_strlen(src);
	while (s < size - 1 && src[s])
	{
		dst[d] = src[s];
		d++;
		s++;
	}
	dst[d] = '\0';
	return (d);
}

static void	ft_init_start_dict(void)
{
	ft_strcpy(g_dict[1], "one");
	ft_strcpy(g_dict[2], "two");
	ft_strcpy(g_dict[3], "three");
	ft_strcpy(g_dict[4], "four");
	ft_strcpy(g_dict[5], "five");
	ft_strcpy(g_dict[6], "six");
	ft_strcpy(g_dict[7], "seven");
	ft_strcpy(g_dict[8], "eight");
	ft_strcpy(g_dict[9], "nine");
	ft_strcpy(g_dict[10], "ten");
	ft_strcpy(g_dict[11], "eleven");
	ft_strcpy(g_dict[12], "twelve");
	ft_strcpy(g_dict[13], "thirteen");
	ft_strcpy(g_dict[14], "fourteen");
	ft_strcpy(g_dict[15], "fifteen");
	ft_strcpy(g_dict[16], "sixteen");
	ft_strcpy(g_dict[17], "seventeen");
	ft_strcpy(g_dict[18], "eighteen");
	ft_strcpy(g_dict[19], "nineteen");
	ft_strcpy(g_dict[20], "twenty");
	ft_strcpy(g_dict[30], "thirty");
	ft_strcpy(g_dict[40], "forty");
	ft_strcpy(g_dict[50], "fifty");
	ft_strcpy(g_dict[60], "sixty");
	ft_strcpy(g_dict[70], "seventy");
	ft_strcpy(g_dict[80], "eighty");
	ft_strcpy(g_dict[90], "ninety");
	ft_strcpy(g_dict[100], "hundred");
	ft_strcpy(g_dict[1000], "thousand");
}

static void	ft_init_dict(void)
{
	int	i;
	int	n;
	int	w;

	ft_init_start_dict();
	i = 21;
	while (i <= 1000)
	{
		w = 0;
		n = i;
		if (n / 1000 > 0)
		{
			ft_strcpy(g_dict[i], g_dict[n / 1000]);
			ft_stracpy(g_dict[i], "thousand");
			w++;
		}
		if (n % 1000 / 100 > 0)
		{
			if (w == 0)
				ft_strcpy(g_dict[i], g_dict[n % 1000 / 100]);
			else
				ft_stracpy(g_dict[i], g_dict[n % 1000 / 100]);
			ft_stracpy(g_dict[i], "hundred");
			w++;
		}
		if (n % 100 / 10 > 0)
		{
			if (w == 0)
				ft_strcpy(g_dict[i], g_dict[n % 100 / 10]);
			else
			{
				ft_stracpy(g_dict[i], "and");
				ft_stracpy(g_dict[i], g_dict[n % 100 / 10]);
				w = 0;
			}
		}
		if (n % 10 > 0)
		{
			if (w > 0)
				ft_stracpy(g_dict[i], "and");
			ft_stracpy(g_dict[i], g_dict[n % 10]);
		}
		i++;
	}
}

int	main(void)
{
	int	i;
	int	sum;

	ft_init_dict();
	i = 1;
	sum = 0;
	while (i <= 1000)
	{
		sum += ft_strlen(g_dict[i]);
		i++;
	}
	printf("sum: %d\n", sum);
	return (0);
}
