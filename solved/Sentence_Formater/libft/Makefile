# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Makefile                                           :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: adelille <adelille@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2020/11/30 19:21:49 by adelille          #+#    #+#              #
#    Updated: 2021/05/10 16:57:04 by adelille         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

NAME =	libft.a
CC = 	clang -Wall -Werror -Wextra
AR =	ar rcs
RM = 	rm -rf
# FLAGS +=	-O2
# FLAGS +=	-g -fsanitize=address

# **************************************************************************** #

MAKEFLAGS += --silent

B =		$(shell tput bold)
BLA =	$(shell tput setaf 0)
RED =	$(shell tput setaf 1)
GRE =	$(shell tput setaf 2)
YEL =	$(shell tput setaf 3)
BLU =	$(shell tput setaf 4)
MAG =	$(shell tput setaf 5)
CYA =	$(shell tput setaf 6)
WHI =	$(shell tput setaf 7)
D =		$(shell tput sgr0)
BEL =	$(shell tput bel)
CLR =	$(shell tput el 1)

# **************************************************************************** #
#	 LIB	#

# LBPATH =	./libft/
# LBNAME =	$(LBPATH)libft.a
# LBINC =		-I$(LBPATH)
# LBM =		libm

# **************************************************************************** #

SRCSPATH =	./srcs/
OBJSPATH =	./objs/
INC =		./includes/

SRCSNAME =	I-O/ft_ps.c I-O/parse/get_next_line.c I-O/parse/get_next_line_utils.c \
				I-O/ft_putchar_fd.c I-O/ft_putendl_fd.c I-O/ft_putnbr_fd.c I-O/ft_putstr_fd.c \
				I-O/ft_pn.c \
				str/ft_atoi.c str/ft_itoa.c str/ft_bzero.c str/ft_split.c str/ft_strchr.c \
				str/ft_strcmp.c str/ft_strdup.c str/ft_strjoin.c str/ft_strlcat.c \
				str/ft_strlcpy.c str/ft_strlen.c str/ft_strmapi.c str/ft_strncmp.c \
				str/ft_strnstr.c str/ft_strrchr.c str/ft_strtrim.c str/ft_substr.c \
				str/ft_tolower.c str/ft_toupper.c \
				nbr/ft_nbrlen.c \
				mem/ft_calloc.c mem/ft_memccpy.c mem/ft_memchr.c mem/ft_memcmp.c \
				mem/ft_memcpy.c mem/ft_memmove.c mem/ft_memset.c \
				lst/ft_lstadd_back.c lst/ft_lstadd_front.c lst/ft_lstclear.c lst/ft_lstdelone.c \
				lst/ft_lstiter.c lst/ft_lstlast.c lst/ft_lstmap.c lst/ft_lstnew.c lst/ft_lstsize.c \
				bool_detect/ft_isalnum.c bool_detect/ft_isalpha.c bool_detect/ft_isascii.c \
				bool_detect/ft_isdigit.c bool_detect/ft_isprint.c

SRCS = $(addprefix $(SRCSPATH), $(SRCSNAME))
OBJSNAME = $(SRCS:.c=.o)
OBJS = $(addprefix $(OBJSPATH), $(notdir $(OBJSNAME)))

%.o: %.c
	$(CC) $(BUFFER) -I$(INC) -c $< -o $(OBJSPATH)$(notdir $@)

# *************************************************************************** #

all:		$(NAME)

$(NAME):	objs_dir $(OBJSNAME) #lib
	@$(AR) $(NAME) $(OBJS)
	#@$(CC) $(FLAGS) $(OBJS) $(LBNAME) -o $(NAME)
	@echo "$(B)$(MAG)$(NAME) compiled.$(D)"

objs_dir:
	@mkdir $(OBJSPATH) 2> /dev/null || true
	
#$(LBM):
#	@make -C $(LBPATH)

#lib:		$(LIBFTM)
#	@echo "$(B)$(MAG)$(BEL)Libft compiled.$(D)"

clean:
	@$(RM) $(OBJSPATH)
# @make clean -C $(LBPATH)
	@echo "$(B)Cleared.$(D)"


fclean:		clean
	@$(RM) $(OBJSPATH)
	@$(RM) $(NAME)
# @make fclean -C $(LBPATH)

re:			fclean all

.PHONY: all clean fclean re objs_dir #lib

# **************************************************************************** #
