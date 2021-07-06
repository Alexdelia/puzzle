/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Sudoku_Validator.c                                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/07/06 17:03:46 by adelille          #+#    #+#             */
/*   Updated: 2021/07/06 17:04:12 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

#define TRUE    1
#define FALSE   0

void    ft_print_count(int count[10])
{
    int b;
    int i;

    i = 1;
    b = FALSE;
    while (i < 10)
    {
        if (count[i] != 1)
            b = TRUE;
        i++;
    }
    if (b == FALSE)
        return ;
    i = 0;
    while (i < 10)
    {
        fprintf(stderr, "|%d", count[i]);
        i++;
    }
    fprintf(stderr, "|\n");
}

void    ft_print_grid(int grid[9][9])
{
    int x;
    int y;

    x = 0;
    while (x < 9)
    {
        y = 0;
        while (y < 9)
        {
            fprintf(stderr, "|%d", grid[x][y]);
            y++;
        }
        fprintf(stderr, "|\n");
        x++;
    }
}

void    ft_count_bzero(int *count)
{
    int i;

    count[0] = 2;
    i = 1;
    while (i < 10)
    {
        count[i] = 0;
        i++;
    }
}

int ft_check_x(int grid[9][9])
{
    int x;
    int y;
    int count[10];

    y = 0;
    while (y < 9)
    {
        ft_count_bzero(count);
        x = 0;
        while (x < 9)
        {
            count[grid[x][y]]++;
            if (count[grid[x][y]] > 1)
                return (FALSE);
            x++;
        }
        //ft_print_count(count);
        y++;
    }
    return (TRUE);
}

int ft_check_y(int grid[9][9])
{
    int x;
    int y;
    int count[10];

    x = 0;
    while (x < 9)
    {
        ft_count_bzero(count);
        y = 0;
        while (y < 9)
        {
            count[grid[x][y]]++;
            if (count[grid[x][y]] > 1)
                return (FALSE);
            y++;
        }
        //ft_print_count(count);
        x++;
    }
    return (TRUE);
}

int ft_check_square(int grid[9][9])
{
    int x;
    int y;
    int count[10];
    int c;

    x = 0;
    while (x < 9)
    {
        y = 0;
        while (y < 9)
        {
            ft_count_bzero(count);
            count[grid[x][y]]++; count[grid[x + 1][y]]++; count[grid[x + 2][y]]++;
            count[grid[x][y + 1]]++; count[grid[x + 1][y + 1]]++; count[grid[x + 2][y + 1]]++;
            count[grid[x][y + 2]]++; count[grid[x + 1][y + 2]]++; count[grid[x + 2][y + 2]]++;
            c = 1;
            while (c < 10)
            {
                if (count[c] != 1)
                    return (FALSE);
                c++;
            }
            //ft_print_count(count);
            y += 3;
        }
        x += 3;
    }
    return (TRUE);
}

int main()
{
    int grid[9][9];
    for (int i = 0; i < 9; i++) {
        for (int j = 0; j < 9; j++) {
            scanf("%d", &grid[i][j]);
        }
    }

    //ft_print_grid(grid);

    if (ft_check_x(grid) == FALSE
            || ft_check_y(grid) == FALSE
            || ft_check_square(grid) == FALSE)
        printf("false\n");
    else
        printf("true\n");

    return 0;
}
