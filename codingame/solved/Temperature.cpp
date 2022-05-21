/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Temperature.cpp                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/05/21 10:09:15 by adelille          #+#    #+#             */
/*   Updated: 2022/05/21 10:09:23 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <iostream>
#include <string>
#include <vector>
#include <algorithm>

using namespace std;

int main(void)
{
    int n,c,t;
    
    cin >> n;
    
    c = n * 10001;
    while(n)
    {
        cin >> t;
        if(abs(t) < abs(c))
            c = t;
        if(abs(t) == abs(c) && t > c)
            c = t;
        n--;
    }

    cout << c;
}
