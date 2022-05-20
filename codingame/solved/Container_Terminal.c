/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Container_Terminal.c                               :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/21 01:27:29 by adelille          #+#    #+#             */
/*   Updated: 2021/05/21 01:27:33 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

#include <unistd.h>

/*
**	Codingame Puzzle
*/

char	ft_last_char(char *stack)
{
	int	i;
	
	if (stack[0] == '~')
		return ('~');
	i = 0;
	while (stack[i] && stack[i] != '~')
		i++;
	return (stack[i - 1]);
}

int	ft_last_int(char *stack)
{
	int	i;
	
	if (stack[0] == '~')
		return (0);
	i = 0;
	while (stack[i] && stack[i] != '~')
		i++;
	return (i);
}

int	ft_solve(char *line)
{
	// don't mind my un-optimised for memory tab of stack
	char	t[26][501];
	int		i;
	int		y;
	int		l;

	i = 0;
	while (i < 26)
	{
		y = 0;
		while (y < 501)
		{
			//write(2, "-", 1);
			t[i][y] = '~';
			y++;
		}
		//write(2, "\n", 1);
		i++;
	}

    //fprintf(stderr, "|%s|\n", line);
	l = 0;
	while (line[l])
	{
		i = 0;
		while (t[i] && ft_last_char(t[i]) < line[l])
			i++;
		t[i][ft_last_int(t[i])] = line[l];
		l++;
	}

	//debug
	write(2, "Stack:\n", 7);
	i = 0;
	while (i < 26 && t[i][0] != '~')
	{
		y = 0;
		write(2, "|", 1);
		while (y < 501 && t[i][y] != '~')
		{
			write(2, &t[i][y], 1);
			y++;
		}
		write(2, "|\n", 2);
		i++;
	}

	i = 0;
	while (t[i] && t[i][0] != '~')
		i++;
	return (i);
}

int main()
{
    int N;
    scanf("%d", &N); fgetc(stdin);
    for (int i = 0; i < N; i++) {
        char	line[501];
        scanf("%[^\n]", line); fgetc(stdin);
		
		printf("%d\n", ft_solve(line));
    }
    return 0;
}
