/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   fo.h                                               :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/31 17:12:59 by adelille          #+#    #+#             */
/*   Updated: 2021/05/31 19:39:01 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#ifndef FO_H
# define FO_H

# include "../libft/includes/libft.h"
# include <sys/types.h>
# include <sys/stat.h>
# include <time.h>
# include <string.h>
# include <fcntl.h>

# define DEFAULT	0
# define UWU		1

typedef struct	s_arg
{
	char		*str;
	char		*res;
	char		*path;
	int			file;
	int			type;
	int			intensity;
}				t_arg;

int		ft_arg(t_arg *a, int ac, char **av);
void	ft_formate(t_arg *a);
void	ft_output(t_arg *a);

int		ft_free_arg(t_arg *a);

#endif
