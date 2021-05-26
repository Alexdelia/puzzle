/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   1D_Spreadsheet.c                                   :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/26 20:53:52 by adelille          #+#    #+#             */
/*   Updated: 2021/05/26 20:53:54 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

//	One of my favorite

# define TRUE   1
# define FALSE  0

typedef struct  s_1d
{
    char    op[6];
    char    a1[7];
    char    a2[7];
    int     i1;
    int     i2;
    int     res;
    int     s;
}               t_1d;

void    ft_resolve(int size, t_1d *d, int index)
{
    // look if it has been solved before
    if (d[index].s == TRUE)
        return ;
    
    // convert a1
    if (d[index].a1[0] == '$')
    {
        // do recursion to solve d[$x]
        ft_resolve(size, d, atoi(&d[index].a1[1]));
        d[index].i1 = d[atoi(&d[index].a1[1])].res;
    }
    else
        d[index].i1 = atoi(d[index].a1);
    // convert a2
    if (d[index].a2[0] == '$')
    {
        // do recursion to solve d[$x]
        ft_resolve(size, d, atoi(&d[index].a2[1]));
        d[index].i2 = d[atoi(&d[index].a2[1])].res;
    }    
    else
        d[index].i2 = atoi(d[index].a2);
    

    // convert res with VALUE operator
    if (strcmp(d[index].op, "VALUE") == 0)
        d[index].res = d[index].i1;
    // convert res with ADD operator
    else if (strcmp(d[index].op, "ADD") == 0)
        d[index].res = d[index].i1 + d[index].i2;
    // convert res with SUB operator
    else if (strcmp(d[index].op, "SUB") == 0)
        d[index].res = d[index].i1 - d[index].i2;
    // convert res with MULT operator
    else if (strcmp(d[index].op, "MULT") == 0)
        d[index].res = d[index].i1 * d[index].i2;
    
    // change solved as TRUE
    d[index].s = TRUE;
    return ;
}

int main()
{
    int N;
    scanf("%d", &N);
    t_1d    d[N];

    for (int i = 0; i < N; i++) {
        scanf("%s%s%s", d[i].op, d[i].a1, d[i].a2);
        d[i].s = FALSE;
        d[i].i1 = 0;
        d[i].i2 = 0;
        d[i].res = 0;
    }

    int i;

    i = 0;
    while (i < N)
    {
        ft_resolve(N, &d, i);
        i++;
    }

    for (int i = 0; i < N; i++) {
        printf("%d\n", d[i].res);
    }

    return 0;
}
