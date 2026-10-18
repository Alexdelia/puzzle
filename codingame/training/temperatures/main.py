# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Temperature.py                                     :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: adelille <adelille@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2022/05/21 10:15:49 by adelille          #+#    #+#              #
#    Updated: 2022/05/21 10:15:56 by adelille         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #


# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

n = int(input())  # the number of temperatures to analyse
c = n * 9999
for i in input().split():
	# t: a temperature expressed as an integer ranging from -273 to 5526
	t = int(i)
	if abs(t) < abs(c):
		c = t
	if abs(t) == abs(c) and t > c:
		c = t

print(c)
