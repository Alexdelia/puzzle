#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>

/*
**	Codingame Puzzle
*/

typedef struct	s_index
{
	int			l;
	int			i;
}				t_i;

int	ft_ps(const char *str)
{
	int	i;

	i = 0;
	while (str[i])
		i++;
	write(1, str, i);
	return (i);
}

void	ft_pt(int tab)
{
	while (tab > 0)
	{
		write(1, "    ", 4);
		tab--;
	}
}

t_i	ft_quote(int N, char cgxline[N][1001], t_i i)
{
	write(1, "\'", 1);
	i.i++;
	while (i.l < N && cgxline[i.l])
	{
		while (cgxline[i.l][i.i] && cgxline[i.l][i.i] != '\'')
		{
			write(1, &cgxline[i.l][i.i], 1);
			i.i++;
		}
		if (cgxline[i.l][i.i] == '\'')
		{
			write(1, "\'", 1);
			return (i);
		}
		i.i = 0;
		i.l++;
	}
	return (i);
}

int main()
{
    int N;
    scanf("%d", &N); fgetc(stdin);
    char cgxline[N][1001];
    for (int i = 0; i < N; i++) {
        scanf("%[^\n]", cgxline[i]); fgetc(stdin);
    }
	t_i	i;
	int	t;

	i.l = 0;
	t = 0;
	while (i.l < N && cgxline[i.l])
	{
		i.i = 0;
		while (i.i <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i])
		{
			while (i.i <= (int)strlen(cgxline[i.l]) &&
					(cgxline[i.l][i.i] == ' ' || cgxline[i.l][i.i] == '\t' || cgxline[i.l][i.i] == '\n'))
				i.i++;
			if (i.i <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i] == '(')
			{
				t++;
				ft_ps("(\n");
				ft_pt(t);
			}
			else if (i.i <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i] == ')')
			{
				if (i.i - 1 >= 0 && cgxline[i.l][i.i - 1] != ')' && cgxline[i.l][i.i] - 1 != '(')
				{
					t--;
					ft_ps("\n");
					ft_pt(t);
				}
				ft_ps(")");
				if (i.i + 1 <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i + 1] && cgxline[i.l][i.i + 1] == ';')
				{
					i.i++;
					ft_ps(";\n");
				}
				else
					ft_ps("\n");
				t--;
				ft_pt(t);
			}
			else if (i.i <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i] == ';')
			{
				ft_ps(";\n");
				ft_pt(t);
			}
			else if (i.i <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i] == '\'')
				i = ft_quote(N, cgxline, i);
			else if (i.i <= (int)strlen(cgxline[i.l]) && cgxline[i.l][i.i]) 
				write(1, &cgxline[i.l][i.i], 1);
			i.i++;
		}
		i.l++;
	}
	if (cgxline[i.l - 1][i.i - 1] != ')')
		ft_ps("\n");
    return 0;
}
