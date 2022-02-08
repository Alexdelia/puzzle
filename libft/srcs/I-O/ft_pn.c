/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_pn.c                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/04/13 15:04:32 by adelille          #+#    #+#             */
/*   Updated: 2021/11/18 16:13:23 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "../../includes/libft.h"

int	ft_pn(int nbr)
{
	ft_putnbr_fd(nbr, STDOUT);
	return (ft_nbrlen(nbr));
}

int	ft_pnc(int nbr, char *color)
{
	write(STDOUT, color, ft_strlen(color));
	ft_putnbr_fd(nbr, STDOUT);
	write(STDOUT, DEF, ft_strlen(DEF));
	return (ft_strlen(color) + ft_nbrlen(nbr) + ft_strlen(DEF));
}

int	ft_pnerc(int nbr, char *color)
{
	write(STDERR, color, ft_strlen(color));
	ft_putnbr_fd(nbr, STDERR);
	write(STDERR, DEF, ft_strlen(DEF));
	return (ft_strlen(color) + ft_nbrlen(nbr) + ft_strlen(DEF));
}
