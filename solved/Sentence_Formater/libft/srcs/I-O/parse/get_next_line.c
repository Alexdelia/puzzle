/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   get_next_line.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/11/19 11:45:40 by adelille          #+#    #+#             */
/*   Updated: 2021/03/30 14:17:04 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "get_next_line.h"

int	check_newline(char *store)
{
	int	i;

	i = 0;
	while (store[i])
	{
		if (store[i] == '\n')
			return (i);
		i++;
	}
	return (-1);
}

int	sub_line_and_dup_store(char **store, char **line, int i)
{
	char	*tmp;

	*line = gnl_ft_substr(*store, 0, i);
	tmp = gnl_ft_strdup(&(*store)[i + 1]);
	free(*store);
	*store = tmp;
	return (1);
}

int	get_next_line_end(char **store, char **line)
{
	int		i;

	if (*store == 0)
	{
		*line = gnl_ft_strdup("");
		return (0);
	}
	if (*store)
	{
		i = check_newline(*store);
		if (i >= 0)
			return (sub_line_and_dup_store(store, line, i));
	}
	else if (*store[0] != '\0')
	{
		*line = gnl_ft_strdup(*store);
		free(*store);
		*store = 0;
		return (0);
	}
	*line = gnl_ft_strdup("");
	free(*store);
	*store = 0;
	return (0);
}

int	get_next_line(int fd, char **line)
{
	char		buffer[BUFFER_SIZE + 1];
	static char	*store[OPEN_MAX];
	ssize_t		read_size;
	int			i;

	if (!line || fd < 0 || fd >= OPEN_MAX || BUFFER_SIZE <= 0 )
		return (-1);
	read_size = 1;
	while (read_size > 0)
	{
		read_size = read(fd, buffer, BUFFER_SIZE);
		if (read_size > 0)
		{
			buffer[read_size] = '\0';
			store[fd] = gnl_ft_strjoin(store[fd], buffer);
			i = check_newline(store[fd]);
			if (i >= 0)
				return (sub_line_and_dup_store(&store[fd], line, i));
		}
	}
	if (read_size < 0)
		return (-1);
	return (get_next_line_end(&store[fd], line));
}
