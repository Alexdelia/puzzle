/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Shadows_of_the_Knight-Ep1.c                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/17 19:33:11 by adelille          #+#    #+#             */
/*   Updated: 2021/05/17 19:33:59 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Codingame Puzzle
*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/

//  debug fonction
void    ft_dpd(char dir[4])
{
    int i;

    i = 0;
    fprintf(stderr, "\n");
    while (i < 4)
    {
        if (dir[i])
            fprintf(stderr, "dir[%d]->|%c|\n", i, dir[i]);
        else
            fprintf(stderr, "dir[%d]->|_|\n", i);
        i++;
    }
    fprintf(stderr, "\n");
}

typedef struct  s_available
{
    int minX;
    int maxX;
    int minY;
    int maxY;
    int X0;
    int Y0;
}               t_ava;

//  debug fonction
void    ft_dpminmax(t_ava a)
{
    fprintf(stderr, "minX[%d]\tmaxX[%d]\nminY[%d]\tmaxY[%d]\n",
            a.minX, a.maxX, a.minY, a.maxY);
}

int ft_find_middle(int minZ, int maxZ)
{
    return ((maxZ - minZ) / 2 + minZ);
}

int ft_contain(char dir[4], char c)
{
    int i;
    
    //fprintf(stderr, "|%c%c|\n", dir[0], (dir[1] ? dir[1] : '_'));
    i = 0;
    while (dir[i])
    {
        if (dir[i] == c)
            return (1);
        i++;
    }
    return (0);
}

void    ft_find_available(t_ava *a, char dir[4])
{
    if (ft_contain(dir, 'L') == 1)
        a->maxX = a->X0 - 1;
    if (ft_contain(dir, 'R') == 1)
        a->minX = a->X0 + 1;
    if (ft_contain(dir, 'U') == 1)
        a->maxY = a->Y0 - 1;
    if (ft_contain(dir, 'D') == 1)
        a->minY = a->Y0 + 1;
    if (!dir[1])
    {
        if (dir[0] == 'L' || dir[0] == 'R')
        {
            a->minY = a->Y0;
            a->maxY = a->Y0;
        }
        else if (dir[0] == 'U' || dir[0] == 'D')
        {
            a->minX = a->X0;
            a->maxX = a->X0;
        }
    }
        
}

int main()
{
    t_ava   a;

    // width of the building.
    int W;
    // height of the building.
    int H;
    scanf("%d%d", &W, &H);
    // maximum number of turns before game over.
    int N;
    scanf("%d", &N);
    // position of Batman.
    scanf("%d%d", &a.X0, &a.Y0);

    a.minX = 0;
    a.maxX = W - 1;
    a.minY = 0;
    a.maxY = H - 1;
    // game loop
    while (1) {
        // the direction of the bombs from batman's current location (U, UR, R, DR, D, DL, L or UL)
        char bomb_dir[4];
        scanf("%s", bomb_dir);

        //debug
        //ft_dpd(bomb_dir);

        ft_find_available(&a, bomb_dir);
        //debug
        //ft_dpminmax(a);

        a.X0 = ft_find_middle(a.minX, a.maxX); // ((maxX - minX) / 2 + minX);
        a.Y0 = ft_find_middle(a.minY, a.maxY); // ((maxY - minY) / 2 + minY);

        // the location of the next window Batman should jump to.
        printf("%d %d\n", a.X0, a.Y0);
    }

    return 0;
}
