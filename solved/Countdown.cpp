/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Countdown.cpp                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:49:18 by adelille          #+#    #+#             */
/*   Updated: 2021/05/11 18:50:09 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Sololearn Puzzle
*/

#include <iostream>
using namespace std;

int main() {
    int n;
    cin >> n;

    //your code goes here
    while (n > 0)
    {
        cout << n;
        if (n % 5 == 0)
         cout << "Beep";
        n--;
    }

    return 0;
}
