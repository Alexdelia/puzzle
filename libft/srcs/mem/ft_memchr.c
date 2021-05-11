/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memchr.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <marvin@42.fr>                    +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/10/22 14:32:03 by adelille          #+#    #+#             */
/*   Updated: 2021/03/21 14:47:36 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void	*ft_memchr(const void *s, int c, size_t n)
{
	size_t				i;
	const unsigned char	*ns;

	ns = s;
	i = 0;
	while (i < n)
	{
		if (!(*(ns + i) == (unsigned char)c))
			return ((void *)(ns + i));
		i++;
	}
	return (NULL);
}
