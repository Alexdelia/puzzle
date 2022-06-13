# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Logic_gates.py                                     :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: adelille <adelille@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2022/06/13 21:34:15 by adelille          #+#    #+#              #
#    Updated: 2022/06/13 21:34:23 by adelille         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

import sys
import math

def exe_gate(t, x, y):
    return{
        "AND":  lambda l, m: l == '-' and m == '-',
        "OR":   lambda l, m: l == '-' or m == '-',
        "XOR":  lambda l, m: (l == '-' and m != '-') or (l != '-' and m == '-'),
        "NAND": lambda l, m: not (l == '-' and m == '-'),
        "NOR":  lambda l, m: not (l == '-' or m == '-'),
        "NXOR": lambda l, m: not ((l == '-' and m != '-') or (l != '-' and m == '-')),
    }[t](x, y)

def logic(t, x, y):
    o = ""
    i = 0
    while i < len(x):
        o += ['_', '-'][exe_gate(t, x[i], y[i])]
        i+=1
    return o


d = {}

n = int(input())
m = int(input())

for i in range(n):
    input_name, input_signal = input().split()
    d[input_name] = input_signal

for i in range(m):
    output_name, _type, input_name_1, input_name_2 = input().split()
    
    d[output_name] = logic(_type, d[input_name_1], d[input_name_2])
    
    print(output_name + ' ' + d[output_name])

print(d, file=sys.stderr, flush=True)
