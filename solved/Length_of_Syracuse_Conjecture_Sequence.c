/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Length_of_Syracuse_Conjecture_Sequence.c           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/07/01 22:21:51 by adelille          #+#    #+#             */
/*   Updated: 2021/07/01 22:22:07 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

int ft_syracuse_len(int s)
{
    int len;

    len = 1;
    while (s > 1)
    {
        if (s % 2 == 0)
            s /= 2;
        else
            s = s * 3 + 1;
        len++;
    }
    return (len);
}

int main()
{
    int N;
    scanf("%d", &N);
    for (int p = 0; p < N; p++) {
        int A;
        int B;
        scanf("%d%d", &A, &B);

        int i;
        int val;
        int len;
        int tmp;

        i = A;
        val = A;
        len = 0;
        while (A <= B)
        {
            tmp = ft_syracuse_len(A);
            if (tmp > len)
            {
                len = tmp;
                val = A;
            }
            A++;
        }
        printf("%d %d\n", val, len);
    }

    return 0;
}
