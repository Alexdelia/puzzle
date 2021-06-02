/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_ps.c                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/02/26 13:09:31 by adelille          #+#    #+#             */
/*   Updated: 2021/05/19 19:26:55 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int	ft_ps(char *str)
{
	int	i;

	i = 0;
	while (str[i])
		i++;
	write(STDOUT, str, i);
	return (i);
}

int	ft_psc(char *str, char *color)
{
	write(STDOUT, color, ft_strlen(color));
	write(STDOUT, str, ft_strlen(str));
	write(STDOUT, DEF, ft_strlen(DEF));
	return (ft_strlen(color) + ft_strlen(str) + ft_strlen(DEF));
}

int	ft_pser(char *str)
{
	int	i;

	i = 0;
	while (str[i])
		i++;
	write(STDERR, str, i);
	return (i);
}

int	ft_pserc(char *str, char *color)
{
	write(STDERR, color, ft_strlen(color));
	write(STDERR, str, ft_strlen(str));
	write(STDERR, DEF, ft_strlen(DEF));
	return (ft_strlen(color) + ft_strlen(str) + ft_strlen(DEF));
}
