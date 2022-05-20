/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Code_vs_Zombies.c                                  :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/05/20 19:47:37 by adelille          #+#    #+#             */
/*   Updated: 2022/05/20 19:47:45 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <iostream>
#include <string>
#include <vector>
#include <algorithm>
#include <cmath>

using namespace std;

/**
 * Save humans, destroy zombies!
 **/

double  distance(const int hx, const int hy, const int zx, const int zy)
{
    return (sqrt((zx - hx) * (zx - hx)
        + (zy - hy) * (zy - hy)));
    /* return (sqrt((hx - zx) * (hx - zx)
        + (hy - zy) * (hy - zy))); */
}

int main()
{
    // game loop
    while (1) {
        int x;
        int y;
        cin >> x >> y; cin.ignore();

        double  b_distance = 900000;
        int     b_x = -1;
        int     b_y = -1;

        int human_count;
        cin >> human_count; cin.ignore();
        for (int i = 0; i < human_count; i++) {
            int human_id;
            int human_x;
            int human_y;
            cin >> human_id >> human_x >> human_y; cin.ignore();
            
            double  tmp;
            tmp = distance(x, y, human_x, human_y);
            if (tmp < b_distance)
            {
                b_distance = tmp;
                b_x = human_x;
                b_y = human_y;
            }
        }

        cerr << b_x << ' ' << b_y << endl;

        b_distance = 900000;
        int bz_x = 0;
        int bz_y = 0;

        int zombie_count;
        cin >> zombie_count; cin.ignore();
        for (int i = 0; i < zombie_count; i++) {
            int zombie_id;
            int zombie_x;
            int zombie_y;
            int zombie_xnext;
            int zombie_ynext;
            cin >> zombie_id >> zombie_x >> zombie_y >> zombie_xnext >> zombie_ynext; cin.ignore();

            double  tmp;
            tmp = distance(b_x, b_y, zombie_x, zombie_y);
            if (tmp < b_distance)
            {
                b_distance = tmp;
                bz_x = zombie_xnext;
                bz_y = zombie_ynext;
            }
        }
        
        cout << bz_x << ' ' << bz_y << endl; // Your destination coordinates
    }
}
