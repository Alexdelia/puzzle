/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Next_Growing_number_[unoptimised].c                :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/06/02 16:41:33 by adelille          #+#    #+#             */
/*   Updated: 2021/06/02 16:42:53 by adelille         ###   ########.fr       */
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
static int	ft_abs(int nbr)
{
	if (nbr < 0)
		nbr = -nbr;
	return (nbr);
}

static void	ft_strrev(char *str)
{
	size_t	len;
	size_t	i;
	char	tmp;

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

char	*ft_itoa(int n)
{
	char	*str;
	int		is_neg;
	size_t	len;

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

int ft_is_valid(char *base, char *n)
{
    int i;

    if (strcmp(base, n) == 0)
        return (0);
    i = 1;
    while (n[i])
    {
        if (n[i - 1] > n[i])
            return (0);
        i++;
    }
    return (1);
}

int main()
{
    char *n;
    char *tmp;
    char base[33];

    scanf("%[^\n]", base);
    n = strdup(base);
    tmp = NULL;

    int nb;
    while (ft_is_valid(base, n) == 0)
    {
        // a lot of alocation/dealocation but I don' want to use the stack and use string here
        free(tmp);
        tmp = strdup(n);
        free(n);
        n = ft_itoa(atoi(tmp) + 1);
    }

    printf("%s\n", n);

    return 0;
