/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Lumen.c                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/07/01 23:38:58 by adelille          #+#    #+#             */
/*   Updated: 2021/07/01 23:39:16 by adelille         ###   ########.fr       */
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

#define CANDLE  2
#define LIGHT   1
#define EMPTY   0

void    ft_print_room(int len, int room[len][len])
{
    int x;
    int y;

    fprintf(stderr, "\n  Room:\n");
    x = 0;
    while (x < len)
    {
        y = 0;
        while (y < len)
        {
            //fprintf(stderr, "[%d%d]", x, y);
            fprintf(stderr, "%d ", room[x][y]);
            y++;
        }
        fprintf(stderr, "\n");
        x++;
    }
    fprintf(stderr, "\n");
}

void    ft_light(int len, int room[len][len], int L, int x, int y)
{
    int base_x;
    int base_y;

    //ft_print_room(len, room);
    fprintf(stderr, "Candle process (%d, %d)\n", x, y);
    if (L < 2)
        return ;
    L--;
    base_x = x - L;
    while (base_x <= x + L)
    {
        base_y = y - L;
        while (base_y <= y + L)
        {
            if (base_x >= 0 && base_x < len && base_y >= 0 && base_y < len
                    && room[base_x][base_y] != CANDLE)
                room[base_x][base_y] = LIGHT; // fprintf(stderr, "IN");
            base_y++;
        }
        base_x++;
    }
}

int main()
{
    int N;
    scanf("%d", &N);
    int L;
    scanf("%d", &L);
    int    room[N][N];
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            char cell[4];
            scanf("%s", cell);
            /*fprintf(stderr, "|%s|-|%c|%c|%c|%c|\n",
                    cell, cell[0], cell[1], cell[2], cell[3]);*/
            fprintf(stderr, "%c", cell[0]);
            //fprintf(stderr, "%c[%d%d] ", cell[0], i, j);
            room[i][j] = (cell[0] == 'C' ? CANDLE : EMPTY);
        }
        fprintf(stderr, "\n");
    }
    fprintf(stderr, "\n");
    //ft_print_room(N, room);

    int x;
    int y;

    x = 0;
    while (x < N)
    {
        y = 0;
        while (y < N)
        {
            if (room[x][y] == CANDLE)
                ft_light(N, room, L, x, y);
            y++;
        }
        x++;
    }

    int dark;

    x = 0;
    dark = 0;
    while (x < N)
    {
        y = 0;
        while (y < N)
        {
            if (room[x][y] == 0)
                dark++;
            y++;
        }
        x++;
    }

    ft_print_room(N, room);
    printf("%d\n", dark);

    return 0;
}
