/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memcmp.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <marvin@42.fr>                    +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/10/22 14:43:11 by adelille          #+#    #+#             */
/*   Updated: 2021/02/26 08:00:23 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int	ft_memcmp(const void *s1, const void *s2, size_t n)
{
	size_t				i;
	const unsigned char	*ns1;
	const unsigned char	*ns2;

	ns1 = s1;
	ns2 = s2;
	i = 0;
	while (i < n)
	{
		if (*(ns1 + i) != *(ns2 + i))
			return (*(ns1 + i) - *(ns2 + i));
		i++;
	}
	return (0);
}
