/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Credit_Card_Verifier.c                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/02 17:39:49 by adelille          #+#    #+#             */
/*   Updated: 2021/06/02 17:39:50 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

#define TRUE    1
#define FALSE   0

/*
**  Codingame Puzzle
*/

// card index to int x2
int ft_citi(char card)
{
    return ((card - '0') * 2);
}

void    ft_init_card(char *card, int *n)
{
    // will segfault if input is not int[len] with len >= 8

    n[0] = ft_citi(card[17]);
    n[1] = ft_citi(card[15]);
    n[2] = ft_citi(card[12]);
    n[3] = ft_citi(card[10]);
    n[4] = ft_citi(card[7]);
    n[5] = ft_citi(card[5]);
    n[6] = ft_citi(card[2]);
    n[7] = ft_citi(card[0]);

    n[8] = -1;
}

int ft_odd_sum(char *card)
{
    int n;

    n = 0;
    n += card[18] - '0';
    n += card[16] - '0';
    n += card[13] - '0';
    n += card[11] - '0';
    n += card[8] - '0';
    n += card[6] - '0';
    n += card[3] - '0';
    n += card[1] - '0';
    
    return (n);
}

int ft_is_valid(char *card)
{
    int n[8];
    int n2;
    int n3;
    int i;

    ft_init_card(card, n);
    i = 0;
    n2 = 0;
    while (i < 8)
    {
        if (n[i] >= 10)
            n[i] -= 9;
        n2 += n[i];
        i++;
    }
    n3 = ft_odd_sum(card);
    return (((n2 + n3) % 10 == 0 ? TRUE : FALSE));
}

int main()
{
    int n;
    scanf("%d", &n); fgetc(stdin);
    for (int i = 0; i < n; i++) {
        char card[21];
        scanf("%[^\n]", card); fgetc(stdin);
        printf("%s\n", (ft_is_valid(card) == TRUE ? "YES" : "NO"));
    }

    return 0;
}
