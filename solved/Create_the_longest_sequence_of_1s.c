/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Create_the_longest_sequence_of_1s.c                :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/07/05 23:24:21 by adelille          #+#    #+#             */
/*   Updated: 2021/07/05 23:24:25 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

#define ONE     1
#define ZERO    0
#define NONE    -1
#define BOTH    2

void    ft_print_split(int *s, int size)
{
    int i;
    
    fprintf(stderr, "split with len of %d:\n|", size);
    i = 0;
    while (i < size)
    {
        fprintf(stderr, "%d|", s[i]);
        i++;
    }
    fprintf(stderr, "\n");
}

int ft_detect_same_sign(char b[1000])
{
    int     i;

    if (!b[0])
        return (NONE);
    i = 1;
    while (b[i])
    {
        if (b[i] != b[0])
            return (BOTH);
        i++;
    }
    return ((b[0] == '0' ? ZERO : ONE));
}

int ft_group_num(char b[1000])
{
    int     i;
    int     total;

    i = 0;
    total = 0;
    while (b[i])
    {
        if (b[i] == '0')
        {
            total++;
        }
        i++;
    }
    if (b[i - 1] == '1')
        total++;
    return (total);
}

void    ft_split(int *s, char b[1000])
{
    int     i;
    int     si;
    int     len;

    i = 0;
    si = 0;
    len = 0;
    while (b[i])
    {
        if (b[i] == '0')
        {
            s[si] = len;
            si++;
            len = -1;
        }
        len++;
        i++;
    }
    if (b[i - 1] == '1')
        s[si] = len;
}

int ft_find_longest(int *s, int size)
{
    int i;
    int big;

    i = 1;
    big = -1;
    while (i < size)
    {
        if (s[i] + s[i - 1] + 1 > big)
            big = s[i] + s[i - 1] + 1;
        i++;
    }
    return (big);
}

int main()
{
    char b[1000];
    scanf("%[^\n]", b);

    int same_sign;

    same_sign = ft_detect_same_sign(b);
    if (same_sign == ONE)
        printf("%zu\n", strlen(b));
    else if (same_sign == ZERO)
        printf("1\n");
    else if (same_sign == NONE)
        printf("0\n");
    else
    {
        int size;

        size = ft_group_num(b);

        int s[size];

        ft_split(s, b);
        ft_print_split(s, size);

        printf("%d\n", ft_find_longest(s, size));
    }
    return 0;
}
