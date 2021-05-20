#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

char	ft_last_char(char *stack)
{
	int	i;
	
	if (stack[0] == '~')
		return ('~');
	i = 0;
	while (stack[i] && stack[i] != '~')
		i++;
	return (stack[i - 1]);
}

int	ft_last_int(char *stack)
{
	int	i;
	
	if (stack[0] == '~')
		return (0);
	i = 0;
	while (stack[i] && stack[i] != '~')
		i++;
	return (i - 1);
}

int	ft_solve(char *line)
{
	// don't mind my un-optimised for memory tab of stack
	char	t[26][501];
	int		i;
	int		y;
	int		l;

	i = 0;
	y = 0;
	while (t[i])
	{
		while (t[i][y])
		{
			t[i][y] = '~';
			y++;
		}
		i++;
	}

    fprintf(stderr, "|%s|\n", line);
	i = 0;
	l = 0;
	while (line[l])
	{
		while (t[i] && ft_last_char(t[i]) >= line[l])
		{
			t[i][ft_last_int(t[i]) + 1] = line[l];
		}
		l++;
	}

	i = 0;
	while (t[i] && t[i][0] != '~')
		i++;
	return (i);
}

int main()
{
    int N;
    scanf("%d", &N); fgetc(stdin);
    for (int i = 0; i < N; i++) {
        char	line[501];
        scanf("%[^\n]", line); fgetc(stdin);
		
		if (i > 0)
			printf(" ");
		printf("%d", ft_solve(line));
    }
    printf("\n");
    return 0;
}
