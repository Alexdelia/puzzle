/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_output.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/31 18:52:13 by adelille          #+#    #+#             */
/*   Updated: 2021/05/31 19:03:47 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "fo.h"

void	ft_output(t_arg *a)
{
	int	fd;

	fd = STDOUT;
	if (a->file == TRUE)
	{
		if ((fd = open(a->path, O_WRONLY | O_TRUNC | O_CREAT, 0664)) <= 0)
		{
			ft_pserc("Error: Unable to open/write/trunc/create ", RED);
			ft_pserc(a->path, RED);
			ft_pser("\n");
			return ;
		}
	}
	ft_putstr_fd(a->res, fd);
	write(fd, "\n", 1);
	if (fd != STDOUT)
		close(fd);
}
