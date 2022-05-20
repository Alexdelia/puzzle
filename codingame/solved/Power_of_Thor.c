/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   Power_of_Thor.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/05/11 18:54:56 by adelille          #+#    #+#             */
/*   Updated: 2021/05/11 18:56:11 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

/*
**	Codingame Puzzle
*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

//	Very simple unelegant solution.

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 * ---
 * Hint: You can use the debug stream to print initialTX and initialTY, if Thor seems not follow your orders.
 **/

int main()
{
    // the X position of the light of power
    int light_x;
    // the Y position of the light of power
    int light_y;
    // Thor's starting X position
    int initial_tx;
    // Thor's starting Y position
    int initial_ty;
    scanf("%d%d%d%d", &light_x, &light_y, &initial_tx, &initial_ty);

    int diff_x;
    int diff_y;

    diff_x = light_x - initial_tx;
    diff_y = light_y - initial_ty;
    int turn;
    // game loop
    while (1) {
        // The remaining amount of turns Thor can move. Do not remove this line.
        int remaining_turns;
        scanf("%d", &remaining_turns);

        // Write an action using printf(). DON'T FORGET THE TRAILING \n
        // To debug: fprintf(stderr, "Debug messages...\n");

        if (diff_x < 0 && diff_y < 0)
        {
            diff_x++;
            diff_y++;
            printf("NW\n");
        }
        else if (diff_x > 0 && diff_y > 0)
        {
            diff_x--;
            diff_y--;
            printf("SE\n");
        }
        else if (diff_x < 0 && diff_y > 0)
        {
            diff_x++;
            diff_y--;
            printf("SW\n");
        }
        else if (diff_x > 0 && diff_y < 0)
        {
            diff_x--;
            diff_y++;
            printf("NE\n");
        }
        else if (diff_x < 0)
        {
            diff_x++;
            printf("W\n");
        }
        else if (diff_x > 0)
        {
            diff_x--;
            printf("E\n");
        }
        else if (diff_y < 0)
        {
            diff_y++;
            printf("N\n");
        }
        else if (diff_y > 0)
        {
            diff_y--;
            printf("S\n");
        }

        remaining_turns--;
        fprintf(stderr, "remaining turn = %d\tdiff_x = %d\tdiff_y = %d\n", remaining_turns, diff_x, diff_y);
        turn = remaining_turns;
        if ((diff_x == 0 && diff_y == 0) || remaining_turns == 0)
            break;

        // A single line providing the move to be made: N NE E SE S SW W or NW
        //printf("SE\n");
    }

    fprintf(stderr, "out of while, remaining turn = %d\n", turn);

    return 0;
}
