#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

typedef struct s_dict
{
	int     c;
	char    b[5001];
}           t_dict;

int    ft_decode(char s[5001], int *i, int n, t_dict d[n])
{
	int init_i;
	int di;
	int b;

	init_i = *i;
	di = 0;
	while (di < n)
	{
		b = 0;
		while (d[di].b[b] == s[*i])
		{
			b++;
			*i += 1;
		}
		if (!d[di].b[b])
			return (d[di].c);
		di++;
		*i = init_i;
	}
	/*fprintf(stderr,
	  "If you see this line, there are no string output possible for this input\n");*/
	return (-1);
}

int main()
{
	int n;
	scanf("%d", &n);
	char    s[5001];
	t_dict  d[n];
	// in case there only one char
	if (n == 1)
	{
		char    b[5001];
		int     c;
		scanf("%s%d", b, &c);
		scanf("%s", s);
		if (strcmp(s, b) == 0)
		{
			printf("%c\n", (char)c);
			return (0);
		}
		strcpy(d[0].b, b);
		d[0].c = c;
	}
	else
	{
		for (int i = 0; i < n; i++) {
			scanf("%s%d", d[i].b, &d[i].c);
		}
		scanf("%s", s);
	}

	char    output[5001];
	int     oi;
	int     tmp;
	int     size;
	int     i;

	i = 0;
	oi = 0;
	size = strlen(s);
	while (i < size)
	{
		tmp = ft_decode(s, &i, n, d);
		if (tmp == -1)
		{
			printf("DECODE FAIL AT INDEX %i\n", i);
			return (0);
		}
		output[oi] = (char)tmp;
		oi++;
	}
	printf("%s\n", output);

	return 0;
}
