/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   36:Double-base_palindromes.c                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/20 18:59:22 by adelille          #+#    #+#             */
/*   Updated: 2022/04/02 18:47:15 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define LIMIT	1000000

#define TRUE	1
#define FALSE	0

// ft_itoa_base from https://github.com/JacobSmolii/42_exam_for_beginers/blob/e7b208c0c07e28a4749f86eb87ca0e147a7f7716/level_5/ft_itoa_base/ft_itoa_base.c
// was lazy to recode it and didn't find my old one

int	get_length(int nbr, int base)
{
	int	len;

	len = 1;
	if (nbr < 0)
	{
		if (base == 10)
			len++;
		nbr = nbr * (-1);
	}
	if (nbr > 0)
	{
		while (nbr >= base)
		{
			nbr = nbr / base;
			len++;
		}
	}
	return (len);
}

char	get_char(int i)
{
	char	*str;

	str = "0123456789ABCDEF";
	return (str[i]);
}

char	*ft_itoa_base(int value, int base)
{
	int				i;
	int				len;
	int				flag;
	unsigned int	val;
	unsigned int	bas;
	char			*str;

	if (value == -2147483648)
		return ("-2147483648");
	val = value;
	bas = base;
	len = get_length(value, base);
	str = (char *)malloc(sizeof(char) * (len + 1));
	i = len;
	str[i] = '\0';
	i--;
	flag = 0;

	if (value < 0)
	{
		if (bas == 10)
			flag = 1;
		val = value * (-1);
	}

	if (base >= 2 && base <= 16)
	{
		while (i >= 0)
		{
			str[i] = get_char(val % bas);
			val /= bas;
			i--;
		}
	}
	if (flag == 1)
		str[0] = '-';
	return (str);
}

static int	ft_is_palindrome(char *str)
{
	int	size;
	int	i;
	
	i = 0;
	size = strlen(str) - 1;
	while (i < size)
	{
		if (str[i] != str[size])
			return (FALSE);
		i++;
		size--;
	}
	return (TRUE);
}

static int	ft_both_palindrome(int n)
{
	char	*deci;
	char	*bina;
	int		res;

	res = FALSE;
	deci = ft_itoa_base(n, 10);
	bina = ft_itoa_base(n, 2);
	if (ft_is_palindrome(deci) == TRUE
		&& ft_is_palindrome(bina) == TRUE)
		res = TRUE;
	free(deci);
	free(bina);
	return (res);
}

int	main(void)
{
	int		n;
	int		i;
	long	sum;

	sum = 0;
	n = 1;
	i = 0;
	while (n < LIMIT)
	{
		if (ft_both_palindrome(n) == TRUE)
		{
			sum += n;
			i++;
		}
		n++;
	}
	printf("Sum of palindromes: %ld (%d)\n", sum, i);
	return (0);
}
