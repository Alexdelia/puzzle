# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    56:Powerful_digit_sum.py                           :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: adelille <adelille@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2022/05/18 19:05:10 by adelille          #+#    #+#              #
#    Updated: 2022/05/18 19:05:46 by adelille         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

print(max(sum(map(int, str(a**b))) for a in range(100) for b in range(100)))