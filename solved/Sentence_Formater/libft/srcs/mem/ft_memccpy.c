/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memccpy.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <marvin@42.fr>                    +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/10/20 13:59:03 by adelille          #+#    #+#             */
/*   Updated: 2021/02/26 07:32:45 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void	*ft_memccpy(void *dest, const void *src, int c, size_t n)
{
	size_t				i;
	unsigned char		*ndest;
	const unsigned char	*nsrc;

	i = 0;
	ndest = dest;
	nsrc = src;
	while (i < n)
	{
		*(ndest + i) = *(nsrc + i);
		if (*(nsrc + i) == (unsigned char)c)
			return (ndest + i + 1);
		i++;
	}
	return (NULL);
}
