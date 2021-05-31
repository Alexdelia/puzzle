/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memcpy.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <marvin@42.fr>                    +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/10/20 13:53:20 by adelille          #+#    #+#             */
/*   Updated: 2021/02/26 07:54:11 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void	*ft_memcpy(void *dest, const void *src, size_t n)
{
	size_t		i;
	char		*ndest;
	const char	*nsrc;

	ndest = dest;
	nsrc = src;
	if (!ndest && !nsrc)
		return (NULL);
	i = 0;
	while (i < n)
	{
		*(ndest + i) = *(nsrc + i);
		i++;
	}
	return (ndest);
}
