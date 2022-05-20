/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Stock_Exchange_Losses.c                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/29 15:56:18 by adelille          #+#    #+#             */
/*   Updated: 2021/06/29 15:56:53 by adelille         ###   ########.fr       */
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
    int n;
    scanf("%d", &n);
    int v[n];
    for (int i = 0; i < n; i++) {
        scanf("%d", &v[i]);
    }

    int start;
    int drop;
    int i;

    start = v[0];
    drop = 0;
    i = 0;

    while (i < n)
    {
        if (v[i] > start)
            start = v[i];
        if (start - v[i] > drop)
            drop = start - v[i];
        i++;
    }

    printf("%d\n", -drop);

    return 0;
}
