/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   color.h                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: adelille <adelille@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2021/11/25 19:01:01 by adelille          #+#    #+#             */
/*   Updated: 2021/11/28 11:51:54 by adelille         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#ifndef COLOR_H
# define COLOR_H

# define C_RESET		"\033[0m"

# define C_BOLD			"\033[1m"
# define C_DIM			"\033[2m"
# define C_ITALIC		"\033[3m"
# define C_UNDERLINE	"\033[4m"
# define C_FLASH		"\033[5m"
# define C_INVERSE		"\033[7m"
# define C_CROSS		"\033[9m"

# define C_BLACK	"\033[30m"
# define C_RED		"\033[31m"
# define C_GREEN	"\033[32m"
# define C_YELLOW	"\033[33m"
# define C_BLUE		"\033[34m"
# define C_MAGENTA	"\033[35m"
# define C_CYAN		"\033[36m"
# define C_WHITE	"\033[37m"
# define C_GRAY		"\033[90m"

# define BWHI	"\033[1;37m"
# define MAG	"\033[0;35m"
# define BMAG	"\033[1;35m"
# define IMAG	"\033[3;35m"
# define BIMAG	"\033[1;3;35m"
# define RED	"\033[1;31m"
# define GRN	"\033[1;32m"
# define SBLU	"\033[5;3;34m"
# define DEF	"\033[0m"

#endif
