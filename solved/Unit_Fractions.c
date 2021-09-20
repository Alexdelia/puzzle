/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Unit_Fractions.c                                   :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/09/17 13:06:43 by adelille          #+#    #+#             */
/*   Updated: 2021/09/17 13:07:45 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

int main()
{
    long    n;
    scanf("%ld", &n);

    long    N;
    long    i;
    long    x;
    long    y;

    N = n*n;
    i = 1;
    x = n + N / i;
    y = n + i;
    while (i < N / 2 + 1 && x >= y)
    {
        if (N % i == 0)
            printf("1/%d = 1/%ld + 1/%ld\n", n, x, y);
        i++;
        x = n + N / i;
        y = n + i;
    }

    return 0;
}
