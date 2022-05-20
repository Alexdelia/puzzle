/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Maximum_sub-sequence.c                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/29 16:29:28 by adelille          #+#    #+#             */
/*   Updated: 2021/06/29 16:29:49 by adelille         ###   ########.fr       */
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
    int N;
    scanf("%d", &N);
    int l[N];
    for (int i = 0; i < N; i++) {
        scanf("%d", &l[i]);
    }

    int actual_min;
    int actual_sequence;
    int min;
    int sequence;
    int i;
    int y;

    min = l[0];
    sequence = 1;
    i = 0;
    
    while (i < N)
    {
        actual_min = l[i];
        actual_sequence = 1;
        y = i + 1;
        while (y < N)
        {
            if (actual_min + actual_sequence == l[y])
                actual_sequence++;
            y++;
        }
        if (actual_sequence > sequence || (actual_sequence == sequence && actual_min < min))
        {
            sequence = actual_sequence;
            min = actual_min;
        }
        i++;
    }

    fprintf(stderr, "min = %d\nsequence = %d\n", min, sequence);
    
    i = 0;
    while (i < sequence)
    {
        printf("%d%c", min, (i + 1 == sequence ? '\n' : ' '));
        i++;
        min++;
    }

    return 0;
}
