# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Don't Panic - Episode 1.py                         :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: adelille <adelille@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2022/09/14 22:48:27 by adelille          #+#    #+#              #
#    Updated: 2022/09/14 22:48:31 by adelille         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

import sys
import math

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

# nb_floors: number of floors
# width: width of the area
# nb_rounds: maximum number of rounds
# exit_floor: floor on which the exit is found
# exit_pos: position of the exit on its floor
# nb_total_clones: number of generated clones
# nb_additional_elevators: ignore (always zero)
# nb_elevators: number of elevators
nb_floors, width, nb_rounds, ef, ep, nb_total_clones, nb_additional_elevators, nb_elevators = [int(i) for i in input().split()]
a=dict()
for i in range(nb_elevators):
    # elevator_floor: floor on which this elevator is found
    # elevator_pos: position of the elevator on its floor
    f, p = [int(j) for j in input().split()]
    print(f, p, file=sys.stderr, flush=True)
    a[f]=p
a[ef]=ep
print(a, file=sys.stderr, flush=True)
# game loop
while True:
    inputs = input().split()
    cf = int(inputs[0])  # floor of the leading clone
    cp = int(inputs[1])  # position of the leading clone on its floor
    d = inputs[2]  # direction of the leading clone: LEFT or RIGHT

    # Write an action using print
    # To debug: print("Debug messages...", file=sys.stderr, flush=True)

    # action: WAIT or BLOCK
    if cf >= 0 and ((a[cf] > cp and d[0] == 'L') or (a[cf] < cp and d[0] == 'R')):
        print("BLOCK")
    else:
        print("WAIT")

