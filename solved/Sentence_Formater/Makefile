# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Makefile                                           :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: adelille <adelille@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2020/11/30 19:21:49 by adelille          #+#    #+#              #
#    Updated: 2021/05/31 19:12:31 by adelille         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

NAME =	Sentence_Formater
CC = 	clang -Wall -Werror -Wextra
AR =	ar rcs
RM = 	rm -rf
# FLAGS +=	-O2
FLAGS =	-g# -fsanitize=address

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

LBPATH =	./libft/
LBNAME =	$(LBPATH)libft.a
LBINC =		-I$(LBPATH)

# **************************************************************************** #

SRCSPATH =	./srcs/
OBJSPATH =	./objs/
INC =		./includes/

SRCSNAME =	main.c \
				ft_arg.c \
				ft_formate.c \
				ft_output.c

SRCS = $(addprefix $(SRCSPATH), $(SRCSNAME))
OBJSNAME = $(SRCS:.c=.o)
OBJS = $(addprefix $(OBJSPATH), $(notdir $(OBJSNAME)))

%.o: %.c
	$(CC) $(FLAGS) $(BUFFER) -I$(INC) -c $< -o $(OBJSPATH)$(notdir $@)

# *************************************************************************** #

all:		$(NAME)

$(NAME):	objs_dir $(OBJSNAME) lib
	#@$(AR) $(NAME) $(OBJS)
	@$(CC) $(FLAGS) $(OBJS) $(LBNAME) -o $(NAME)
	@echo "$(B)$(MAG)$(NAME) compiled.$(D)"

objs_dir:
	@mkdir $(OBJSPATH) 2> /dev/null || true
	
lib:
	@make -C $(LBPATH)

clean:
	@$(RM) $(OBJSNAME)
	@make clean -C $(LBPATH)
	@echo "$(B)Cleared.$(D)"


fclean:		clean
	@$(RM) $(OBJSPATH)
	@$(RM) $(NAME)
	@make fclean -C $(LBPATH)

re:			fclean all

.PHONY: all clean fclean re objs_dir lib

# **************************************************************************** #
