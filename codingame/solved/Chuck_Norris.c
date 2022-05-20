/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Chuck_Norris.c                                     :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:51:33 by adelille          #+#    #+#             */
/*   Updated: 2021/05/11 18:52:43 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Codingame Puzzle
*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

void    ft_init(int *b, int size)
{
    int i;

    i = 0;
    while (i < size)
    {
        b[i] = -1;
        i++;
    }
}

int     ft_strlen(char *str)
{
    int i;

    i = 0;
    while (str[i])
        i++;
    return (i);
}

int     ft_bilen(char c)
{
    int i;
    int n;

    n = (int)c;
    i = 0;
    while (n > 0)
    {
        n /= 2;
        i++;
    }
    return (i);
}

void    ft_atob(int *b, char *str)
{
    int i;
    int bi;
    int n;
    int s;
    int bi_len;

    i = 0;
    bi = 0;
    while (str[i])
    {
        n = str[i];
        fprintf(stderr, "n = %d\tc = %c\n", n, str[i]);
        bi += 7;
        s = 0;
        bi_len = ft_bilen(str[i]);
        fprintf(stderr, "bi_len = %d\n", bi_len);
        while (bi_len > 0)
        {
            b[bi] = n % 2;
            n /= 2;

            fprintf(stderr, "n = %d  \tb[%d] = %d\t(N)\n", n, bi, b[bi]);
            bi--;
            s++;
            bi_len--;
        }
        while (s < 7)
        {
            b[bi] = 0;
            fprintf(stderr, "n = %d  \tb[%d] = %d\t(S)\n", n, bi, b[bi]);
            bi--;
            s++;
        }
        bi += 7;
        fprintf(stderr, "\n");
        i++;
    }
}

void    ft_print_b(int *b, int size)
{
    int i;

    i = 1;
    while (i < size)
    {
        fprintf(stderr, "%d", b[i]);
        i++;
    }
    fprintf(stderr, "\n");
}

int main()
{
    char MESSAGE[101];
    scanf("%[^\n]", MESSAGE);

    int b[880];
    int size;
    int i;

    i = 1;
    size = ft_strlen(MESSAGE) * 7 + 1;
    //fprintf(stderr, "START\n");
    ft_init(b, size);
    ft_atob(b, MESSAGE);
    //fprintf(stderr, "END CONV\nsize = %d\n", size);
    ft_print_b(b, size);
    while (i < size)
    {
        //fprintf(stderr, "IN\n");
        if (b[i] == 0)
        {
            printf("00 ");
            while (b[i] == 0 && i < size)
            {
                fprintf(stderr, "0");
                printf("0");
                i++;
            }
            if (i < size)
                printf(" ");
        }
        else if (b[i] == 1)
        {
            printf("0 ");
            while (b[i] == 1 && i < size)
            {
                fprintf(stderr, "1");
                printf("0");
                i++;
            }
            if (i < size)
                printf(" ");
        }
        else
        {
            fprintf(stderr, "Oh No: %d\tb[%d]\n", b[i], i);
            i++;
        }
    }
    fprintf(stderr, "\nDone\n");

    return 0;
}
