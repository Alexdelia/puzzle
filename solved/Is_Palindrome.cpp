/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Is_Palindrome.cpp                                  :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:42:32 by adelille          #+#    #+#             */
/*   Updated: 2021/05/11 18:43:09 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Sololearn Puzzle
*/

#include <iostream>
#include <stdlib.h>
#include <stdio.h>
using namespace std;

int ft_strlen(char *str)
{
    int i;

    i = 0;
    while (str[i])
       i++;
    return (i);
}

void ft_itoa(char *a, int n)
{
    int i;

    i = 0;
    while (n > 0)
    {
        a[i] = n % 10 + '0';
        i++;
        n /= 10;
    }
    a[i] = '\0';
}

bool isPalindrome(int x) {
    char a[13];
    int size;
    int i;

    //x = 88888;
    ft_itoa(a, x);
    size = ft_strlen(a);
    //cout << "a: " << a << " size: " << size << endl;
    i = 0;
    while (a[i] && i < size / 2)
    {
        //cout << a[i] << " - " << a[size - 1 - i] << endl;
        if (a[i] != a[size - 1 - i])
           return (false);
        i++;
    }
    return (true);
}

int main() {
    int n;
    cin >>n;

    if(isPalindrome(n)) {
        cout <<n<<" is a palindrome";
    }
    else {
        cout << n<<" is NOT a palindrome";
    }
    return 0;
}
