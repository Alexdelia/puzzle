#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

#define	TRUE	1
#define	FALSE	0

typedef struct	s_mime
{
	char			*EXT;
	char			*MIME;
	struct s_mime	*next;
}				t_m;

typedef struct	s_data
{
	t_m			*m;
}				t_d;

int		ft_strlen(const char *str)
{
	int	i;

	i = 0;
	while (str[i])
		i++;
	return (i);
}

char	*ft_strdup(const char *s)
{
	char	*str;
	size_t	i;

	str = (char *)malloc(sizeof(*s) * (ft_strlen(s) + 1));
	if (!str)
		return (NULL);
	i = 0;
	while (s[i])
	{
		str[i] = s[i];
		i++;
	}
	str[i] = '\0';
	return (str);
}

t_m		*ft_mime_new(char EXT[11], char MIME[51])
{
	t_m	*elt;

	elt = (t_m *)malloc(sizeof(*elt));
	if (!elt)
		return (NULL);
	elt->EXT = ft_strdup(EXT);
	elt->MIME = ft_strdup(MIME);
	elt->next = NULL;
	return (elt);
}

t_m		*ft_mime_last(t_m *m)
{
	while (m)
	{
		if (!m->next)
			return (m);
		m = m->next;
	}
	return (m);
}

void	ft_mime_addback(t_m **m, t_m *new)
{
	t_m	*last;

	if (m)
	{
		if (*m)
		{
			last = ft_mime_last(*m);
			last->next = new;
		}
		else
			*m = new;
	}
}

void	ft_mime_freeone(t_m *m)
{
	if (m)
	{
		free(m->EXT);
		free(m->MIME);
		free(m);
	}
}

void	ft_mime_freeall(t_m **m)
{
	t_m	*tmp;

	if (!m || !*m)
		return ;
	while (*m)
	{
		tmp = (*m)->next;
		ft_mime_freeone(*m);
		*m = tmp;
	}
}

char	*ft_extension(char FNAME[257])
{
	int	i;
	int	p;

	i = 0;
	p = FALSE;
	while (FNAME[i])
	{
		if (FNAME[i] == '.')
			p = TRUE;
		i++;
	}
	if (p == FALSE)
		return ("\0");
	if (i > 0)
		i--;
	while (i >= 0 && FNAME[i] != '.')
		i--;
	if (FNAME[i] == '.' && FNAME[i + 1])
		return (&FNAME[i + 1]);
	return ("\0");
}

int		ft_same_letter(char c1, char c2)
{
	if (c1 == c2 || c1 + 32 == c2 || c1 == c2 + 32)
		return (TRUE);
	return (FALSE);
}

int		ft_strcmp(const char *s1, const char *s2)
{
	int	i;

	i = 0;
	while (s1[i] && s2[i])
	{
		if (ft_same_letter(s1[i], s2[i]) == FALSE)
			return (s1[i] - s2[i]);
		i++;
	}
	if (!s1[i] && !s2[i])
		return (0);
	return (s1[i] - s2[i]);
}

char	*ft_mime_search(t_m *m, char *EXT)
{
	while (m)
	{
		if (ft_strcmp(EXT, m->EXT) == 0)
			return (m->MIME);
		m = m->next;
	}
	return ("UNKNOWN");
}

char	*ft_mime(t_d *d, char FNAME[257])
{
	return (ft_mime_search(d->m, ft_extension(FNAME)));
}

void	ft_solve(t_d *d, int N, int Q)
{
    for (int i = 0; i < N; i++) {
        // file extension
        char EXT[11];
        // MIME type.
        char MT[51];
        scanf("%s%s", EXT, MT); fgetc(stdin);
		// filling my dictionary
		if (i == 0)
			d->m = ft_mime_new(EXT, MT);
		else
			ft_mime_addback(&d->m, ft_mime_new(EXT, MT));
		//fprintf(stderr, "%d|%s|%s|\n", i, d->m->EXT, (d->m->next ? d->m->next->EXT : "_"));
    }
	fprintf(stderr, "X|%s|%s|\n", d->m->EXT, (d->m->next ? d->m->next->EXT : "_"));
    for (int i = 0; i < Q; i++) {
        // One file name per line.
        char FNAME[257];
        scanf("%[^\n]", FNAME); fgetc(stdin);
		printf("%s\n", ft_mime(d, FNAME));
    }
	ft_mime_freeall(&d->m);
}
int main()
{
    // Number of elements which make up the association table.
    int N;
    scanf("%d", &N);
    // Number Q of file names to be analyzed.
    int Q;
    scanf("%d", &Q);

	t_d	d;
	ft_solve(&d, N, Q);
    return 0;
}
