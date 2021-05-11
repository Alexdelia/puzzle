/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Ticked_Office.cpp                                  :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:48:13 by adelille          #+#    #+#             */
/*   Updated: 2021/05/11 18:49:04 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Sololearn Puzzle
*/

#include <iostream>
using namespace std;

int main() {
    int ages[5];
    double lowest;
    int x;

    for (int i = 0; i < 5; ++i) {
        cin >> ages[i];
    }
    //your code goes here
    lowest = ages[0];
    x = 0;
    while (x < 5)
    {
        if (ages[x] < lowest)
         lowest = ages[x];
        //cout << ages[x] << endl;
        x++;
    }
    //cout << "low " << lowest << " here " << -(lowest / 100 - 1) << endl;
    if (x == 5)
     cout << 10 * x * -(lowest / 100 - 1);
    else
     cout << 10 * x;
    return 0;
}
