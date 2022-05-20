/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Horse-Racing_Duals.c                               :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/26 19:13:24 by adelille          #+#    #+#             */
/*   Updated: 2021/05/26 19:13:25 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <limits.h>

/*
**	Codingame Puzzle
*/

// unoptimsed and easy to understand solution
/*
int ft_diff(int x, int y)
{
    if (x - y < 0)
        return (-(x - y));
    return (x - y);
}

int ft_closest(int N, int pi[N], int p)
{
    int i;
    int diff;
    int c;

    i = 0;
    diff = INT_MAX;
    while (i < N)
    {
        c = ft_diff(pi[i], p);
        diff = (c > 0 && c < diff ? c : diff);
        i++;
    }
    return (diff);
}
*/

// optimised solution using quick sort
int ft_comp(const void *a,const void *b)
{
    int *x = (int *) a;
    int *y = (int *) b;
    return *x - *y;
}

int ft_closest_search(int size, int pi[size])
{
    int i;
    int diff;

    qsort(pi, size, sizeof(*pi), ft_comp);
    i = 1;
    diff = INT_MAX;
    while (i < size)
    {
        if (pi[i] - pi[i - 1] < diff)
            diff = pi[i] - pi[i - 1];
        i++;
    }
    return (diff);
}

int main()
{
    int N;
    scanf("%d", &N);
    int pi[N];
    for (int i = 0; i < N; i++) {
        scanf("%d", &pi[i]);
    }

    int diff;
    
	// unoptimised and easy to understand solution
    /*
    int i;
    int c;

    diff = INT_MAX;
    i = 0;
    while (i < N)
    {
        c = ft_closest(N, pi, pi[i]);
        diff = (c < diff ? c : diff);
        i++;
    }*/

    // optimised solution using quick sort
    diff = ft_closest_search(N, pi);

    printf("%d\n", diff);

    return 0;
}
