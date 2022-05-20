/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Rock_Paper_Scissors_Lizard_Spock.c                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/23 18:23:39 by adelille          #+#    #+#             */
/*   Updated: 2021/06/23 18:23:40 by adelille         ###   ########.fr       */
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

#define TRUE    1
#define FALSE   0

#define ROCK    0
#define SPOCK   1
#define PAPER   2
#define LIZARD  3
#define SCIZOR  4

typedef struct  s_players
{
    int         num;
    int         sign;
}               t_p;

char ft_itoc(int sign)
{
    char    c[5] = {'R', 'S', 'P', 'L', 'C'};
    return (c[sign]);
}

int ft_ctoi(char sign)
{
    switch(sign)
    {
        case 'R':
            return (ROCK);
        case 'S':
            return (SPOCK);
        case 'P':
            return (PAPER);
        case 'L':
            return (LIZARD);
        case 'C':
            return (SCIZOR);
    }
    fprintf(stderr, "Error: sign not found\n");
    return (-1);
}

t_p ft_round(t_p p1, t_p p2)
{
    int res;

    res = (p1.sign - p2.sign) % 5;
    //fprintf(stderr, "res: %d\n", res);
    if (res >= 3)
        return (p2);
    else if (res >= 1)
        return (p1);
    else if (res <= -3)
        return (p1);
    else if (res <= -1)
        return (p2);

    /*fprintf(stderr, "res: %d | Draw: %d-%c(%d) vs %d-%c(%d)\n",
        res, p1.num, ft_itoc(p1.sign), p1.sign, p2.num, ft_itoc(p2.sign), p2.sign);*/
    if (p1.num < p2.num)
        return (p1);
    else
        return (p2);
}

int ft_find_winner(t_p *p, int size, int print, int winner)
{
    t_p pw[size / 2];
    int i;

    i = 0;
    while (i < size)
    {
        pw[i / 2] = ft_round(p[i], p[i + 1]);
        /*if (print == 0)
            fprintf(stderr, "%c : %d-%c | i = %d | %d-%c vs %d-%c\n",
                (print == 0 ? 'W' : 'S'), pw[i / 2].num, ft_itoc(pw[i / 2].sign), i,
                p[i].num, ft_itoc(p[i].sign), p[i + 1].num, ft_itoc(p[i + 1].sign));*/
        if (winner == pw[i / 2].num && print == TRUE)
        {
            printf("%d", (winner == p[i].num ? p[i + 1].num : p[i].num));
            if (size != 2)
                write(1, " ", 1);
        }
        i += 2;
    }

    /*if (print == 0)
        fprintf(stderr, "NEXT\n");*/
    if (size == 2)
        return (pw[0].num);
    return (ft_find_winner(pw, size / 2, print, winner)); 
}

int main()
{
    int N;
    scanf("%d", &N);
    t_p p[N];
    for (int i = 0; i < N; i++) {
        char SIGNPLAYER[2];
        scanf("%d%s", &p[i].num, SIGNPLAYER);
        p[i].sign = ft_ctoi(SIGNPLAYER[0]);
    }
    // it can be done in one shot, but here it's very simple to understand

    int winner;
    
    // do all match
    winner = ft_find_winner(p, N, FALSE, -1);
    printf("%d\n", winner);

    // redo the matchs following the winner
    ft_find_winner(p, N, TRUE, winner);

    return 0;
}
