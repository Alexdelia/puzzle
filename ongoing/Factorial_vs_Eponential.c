/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Factorial_vs_Eponential.c                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/29 17:25:00 by adelille          #+#    #+#             */
/*   Updated: 2021/06/29 17:25:04 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <math.h>

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/

#define FACTO_LEN   3001

void    ft_fill_factorial(long double *f, int size)
{
    int i;

    f[0] = 1;
    i = 1;
    while (i < size)
    {
        f[i] = f[i - 1] * i;
        i++;
    }
}

int main()
{
    long double factorial[FACTO_LEN];
    ft_fill_factorial(factorial, FACTO_LEN);

    int K;
    scanf("%d", &K);
    int N;
    for (int i = 0; i < K; i++) {
        float A;
        scanf("%f", &A);

        N = 1;
        while (N < FACTO_LEN && pow((double)A, (double)N) >= factorial[N])
            N++;
        
        if (N == FACTO_LEN)
            fprintf(stderr, "N > %d | pow(%f, %d) [%f] >= !%d [%d]\n", FACTO_LEN,
                            A, N, pow((double)A, (double)N), N, factorial[N]);
        printf("%d%c", N, (i + 1 == K ? '\n' : ' '));
    }

    return 0;
}
