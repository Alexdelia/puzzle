/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.c                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/31 17:06:54 by adelille          #+#    #+#             */
/*   Updated: 2021/05/31 19:39:25 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "fo.h"

int	ft_free_arg(t_arg *a)
{
	if (a->str)
		free(a->str);
	if (a->res)
		free(a->res);
	if (a->path)
		free(a->path);
	return (2);
}

static void	ft_init_arg(t_arg *a)
{
	a->str = NULL;
	a->res = NULL;
	a->path = NULL;
	a->file = -1;
	a->type = DEFAULT;
	a->intensity = 0;
}

int	main(int ac, char **av)
{
	t_arg	a;

	if (ac == 1)
		return (ft_pserc("No argument\n", RED) * 0);
	ft_init_arg(&a);
	if (ft_arg(&a, ac, av) == FALSE)
		return (ft_free_arg(&a));
	ft_formate(&a);
	ft_output(&a);
	ft_free_arg(&a);
	return (0);
}
