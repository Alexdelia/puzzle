/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Temperature.c                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:57:17 by adelille          #+#    #+#             */
/*   Updated: 2021/05/11 18:57:48 by adelille         ###   ########.fr       */
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

int main()
{
    // the number of temperatures to analyse
    int n;
    int index;
    scanf("%d", &n);
    float tmp;
    int degree[n];
    float closer;
    int closer_i;

    index = 0;
    for (int i = 0; i < n; i++) {
        // a temperature expressed as an integer ranging from -273 to 5526
        int t;
        scanf("%d", &t);
        degree[i] = t;
    }
    closer_i = 0;
    if (degree[0] < 0)
        closer = -degree[0];
    else
        closer = degree[0];
    while (index < n)
    {
        if (degree[index] < 0)
            tmp = -degree[index] + 0.5;
        else
            tmp = degree[index];
        fprintf(stderr, "degree = %d\ttrans = %f\n", degree[index], tmp);
        fprintf(stderr, "closer = %d\ttrans[%d] = %f\n\n", closer, index, tmp);
        if (closer > tmp)
        {
            closer = tmp;
            closer_i = index;
        }
        index++;
    }

    // Write an answer using printf(). DON'T FORGET THE TRAILING \n
    // To debug: fprintf(stderr, "Debug messages...\n");

    printf("%d\n", degree[closer_i]);

    return 0;
}
