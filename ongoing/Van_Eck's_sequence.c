/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Van_Eck's_sequence.c                               :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/30 19:17:16 by adelille          #+#    #+#             */
/*   Updated: 2021/06/30 19:58:43 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

int ft_seen(int *a, int index)
{
    int to_find;
    int i;

    to_find = a[index];
    i = index - 1;
    while (i >= 0)
    {
        if (a[i] == to_find)
            return (index - i);
        i--;
    }
    return (0);
}

int main()
{
    int A1;
    scanf("%d", &A1);
    int N;
    scanf("%d", &N);

    int	mem[950220] = { 0 };
    int	a[N];
    int	i;
    int	debug_big;

    //fprintf(stderr, "Starting: n0:%d ", A1);
    debug_big = -1;

    a[0] = A1;
    i = 1;
    while (i < N)
    {
		if (mem[a[i - 1]] == 0)
		{
			mem[a[i - 1]] = 1;
			a[i] = 0;
		}
		else
        	a[i] = ft_seen(a, i - 1);
        //fprintf(stderr, "n%d:%d ", i, a[i]);
        //fprintf(stderr, ", %d", a[i]);
        if (a[i] > debug_big)
        {
            debug_big = a[i];
            //fprintf(stderr, "%d\n", debug_big);
			fprintf(stderr, "\r[%d%%]", (i * 100) / N);
        }
        i++;
    }

    fprintf(stderr, "\n\nBiggest: %d\n", debug_big);

    printf("%d\n", a[i - 1]);

    return 0;
}
