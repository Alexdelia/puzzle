/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_arg.c                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/31 17:26:32 by adelille          #+#    #+#             */
/*   Updated: 2021/05/31 19:44:28 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "fo.h"

static int	ft_options(t_arg *a, char **av, int i)
{
	if (strcasecmp(av[i], "-uwu") == 0)
	{
		a->type = UWU;
		if (av[i + 1] && ft_isdigit(av[i + 1][0]))
		{
			a->intensity = ft_atoi(av[i + 1]);
			a->intensity = (a->intensity > 10 ? 10 :
							(a->intensity < 0 ? 0 : a->intensity));
			return (i + 1);
		}
		else
			a->intensity = 7;
	}
	return (i);
}

static int	ft_str(t_arg *a, char *str)
{
	struct stat	stats;
	int			fd;
	char		*line;

	line = NULL;
	if (stat(str, &stats) == -1)
	{
		a->file = FALSE;
		a->str = ft_strdup(str);
		a->res = ft_strdup(str);
		return (TRUE);
	}
	else
	{
		if ((fd = open(str, O_RDONLY)) <= 0)
		{
			ft_pserc("Error: Unable to open ", RED);
			ft_pserc(str, RED);
			ft_pser("\n");
			return (FALSE);
		}
		if (get_next_line(fd, &line) <= 0)
		{
			ft_pserc("Error: ", RED);
			ft_pserc(str, RED);
			ft_pserc(" is empty\n", RED);
			free(line);
			return (FALSE);
		}
		a->file = TRUE;
		a->str = ft_strdup(line);
		a->res = ft_strdup(line);
		a->path = ft_strdup(str);
		free(line);
		close(fd);
	}
	return (TRUE);
}

int	ft_arg(t_arg *a, int ac, char **av)
{
	int			str;
	int			i;

	str = FALSE;
	i = 1;
	(void)ac;
	while (av[i])
	{
		if (av[i][0] == '-')
			i = ft_options(a, av, i);
		else if (str == FALSE)
		{
			if (ft_str(a, av[i]) == FALSE)
				return (FALSE);
			str = TRUE;
		}
		i++;
	}
	if (str == FALSE)
	{
		ft_pserc("No input\n", RED);
		return (FALSE);
	}
	return (TRUE);
}
