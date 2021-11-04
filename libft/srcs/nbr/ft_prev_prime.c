/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_next_prime_ul.c                                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/03 18:10:35 by adelille          #+#    #+#             */
/*   Updated: 2021/11/04 14:05:30 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../../includes/libft.h"

int	ft_prev_prime(int n)
{
	if (n <= 2)
		return (INT_MAX);
	if (n % 2 == 0)
		n--;
	else
		n -= 2;
	while (ft_is_prime(n) == false)
		n -= 2;
	return (n);
}

unsigned long	ft_prev_prime_ul(unsigned long n)
{
	if (n <= 2)
		return (0);
	if (n % 2 == 0)
		n--;
	else
		n -= 2;
	while (ft_is_prime(n) == false)
		n -= 2;
	return (n);
}
