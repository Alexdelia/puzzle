/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   63:Powerful_digit_counts.cpp                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2022/05/18 19:07:44 by adelille          #+#    #+#             */
/*   Updated: 2022/05/18 19:43:29 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include <string>
#include <cmath>
#include <iostream>

int main(void)
{
	unsigned int	pow;
	unsigned int	count;
	unsigned int	base;
	std::string		tmp;

	count = 0;
	pow = 1;
	while (pow < 25)
	{
		base = 2;
		tmp = std::to_string(std::pow(base, pow));
		while (tmp.size() - 7 <= pow)
		{
			if (tmp.size() - 7 == pow)
			{
				count++;
				std::cout << base << '^' << pow << " = " << tmp
					<< "\t(" << tmp.size() - 7 << ')' << std::endl;
			}
			base++;
			tmp = std::to_string(std::pow(base, pow));
		}
		std::cout << std::endl;
		pow++;
	}

	std::cout << count << std::endl;

	return (0);
}