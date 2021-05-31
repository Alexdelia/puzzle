/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   get_next_line.h                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/11/19 13:43:14 by adelille          #+#    #+#             */
/*   Updated: 2021/04/11 18:48:39 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#ifndef GET_NEXT_LINE_H
# define GET_NEXT_LINE_H
# include <stdlib.h>
# include <unistd.h>
# include <limits.h>

# define OPEN_MAX 1024
# define BUFFER_SIZE 1024

int		get_next_line(int fd, char **line);

char	*gnl_ft_strjoin(char *s1, char *s2);
char	*gnl_ft_substr(char const *s, unsigned int start, size_t len);
char	*gnl_ft_strdup(const char *s);

size_t	ft_strlen(const char *str);

#endif
