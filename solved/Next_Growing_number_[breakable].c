/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Next_Growing_number_[breakable].c                  :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/02 17:14:10 by adelille          #+#    #+#             */
/*   Updated: 2021/06/02 17:14:12 by adelille         ###   ########.fr       */
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

// itoa from Alexdelia's 42-Libft
static int      ft_abs(int nbr)
{
        if (nbr < 0)
                nbr = -nbr;
        return (nbr);
}

static void     ft_strrev(char *str)
{
        size_t  len;
        size_t  i;
        char    tmp;

        len = strlen(str);
        i = 0;
        while (i < len / 2)
        {
                tmp = str[i];
                str[i] = str[len - i - 1];
                str[len - i - 1] = tmp;
                i++;
        }
}

char    *ft_itoa(int n)
{
        char    *str;
        int             is_neg;
        size_t  len;

        is_neg = (n < 0);
        str = calloc(11 + is_neg, sizeof(*str));
        if (!str)
                return (NULL);
        if (n == 0)
                str[0] = '0';
        len = 0;
        while (n != 0)
        {
                str[len++] = '0' + ft_abs(n % 10);
                n = (n / 10);
        }
        if (is_neg)
                str[len] = '-';
        ft_strrev(str);
        return (str);
}
// End of itoa from Alexdelia's 42-Libft

int main()
{
    char *n;
    char base[33];

    scanf("%[^\n]", base);
    // it break with already growing number over INT_MAX
    if (strlen(base) < 11)
        n = ft_itoa(atoi(base) + 1);
    else
        n = strdup(base);

    int     i;
    int     y;

    i = 1;
    while (n[i])
    {
        if (n[i] < n[i - 1])
        {
            n[i] = n[i - 1];
            y = i + 1;
            while (n[y])
            {
                n[y] = '0';
                y++;
            }
        }
        i++;
    }

    printf("%s\n", n);
    free(n);

    return 0;
}
