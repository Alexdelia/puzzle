/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   get_next_line_utils.c                              :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2020/11/19 13:42:55 by adelille          #+#    #+#             */
/*   Updated: 2021/03/21 14:51:35 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "get_next_line.h"

char	*gnl_ft_strdup(const char *s)
{
	char	*str;
	int		i;

	str = (char *)malloc(ft_strlen(s) + 1);
	if (!str)
		return (NULL);
	i = -1;
	while (s[++i])
		str[i] = s[i];
	str[i] = '\0';
	return (str);
}

char	*gnl_ft_substr(char const *s, unsigned int start, size_t len)
{
	size_t	i;
	char	*str;

	str = (char *)malloc(len + 1);
	if (!str)
		return (NULL);
	i = 0;
	while (i < len && s[i + start])
	{
		str[i] = s[i + start];
		i++;
	}
	str[i] = '\0';
	return (str);
}

char	*gnl_ft_strjoin(char *s1, char *s2)
{
	int		i;
	int		j;
	char	*str;

	if (!s1)
		return (gnl_ft_strdup(s2));
	str = (char *)malloc(sizeof(char) * (ft_strlen(s1) + ft_strlen(s2) + 1));
	if (!str)
		return (NULL);
	i = -1;
	j = 0;
	while (s1[++i])
		str[i] = s1[i];
	free(s1);
	s1 = NULL;
	while (s2[j])
	{
		str[i] = s2[j];
		i++;
		j++;
	}
	str[i] = '\0';
	return (str);
}
