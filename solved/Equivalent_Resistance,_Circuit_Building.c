#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

typedef struct s_dict
{
    char    name[11];
    int     R;
}           t_dict;

float   ft_parallel(int N, t_dict d[N], char circuit[121], int *i);
float   ft_series(int N, t_dict d[N], char circuit[121], int *i);

int     ft_dict_search(int N, t_dict d[N], char *circuit)
{
    int     i;
    int     size;
    char    name[11];

    i = 0;
    size = strlen(circuit);
    while (i < size && circuit[i] != ' ')
    {
        name[i] = circuit[i];
        i++;
    }
    name[i] = '\0';
    //fprintf(stderr, "n %s\t| %i\t|<%s\n", name, size, circuit);
    i = 0;
    while (i < N)
    {
        if (strcmp(d[i].name, name) == 0)
            return (d[i].R);
        i++;
    }
    return (0);
}

float   ft_parallel(int N, t_dict d[N], char circuit[121], int *i)
{
    float   total;

    total = 0;
    *i += 1;
    while (circuit[*i] != ']')
    {
        if (circuit[*i] == '(')
            total += 1 / ft_series(N, d, circuit, i);
        else if (circuit[*i] == '[')
            total += 1 / ft_parallel(N, d, circuit, i);
        else if (circuit[*i] != ' ')
        {
            total += 1 / (float)ft_dict_search(N, d, &circuit[*i]);
            while (circuit[*i] != ' ')
                *i += 1;
        }
        *i += 1;
    }
    //fprintf(stderr, "P %f\n", 1 / total);
    return (1 / total);
}

float   ft_series(int N, t_dict d[N], char circuit[121], int *i)
{
    float   total;

    total = 0;
    *i += 1;
    while (circuit[*i] != ')')
    {
        if (circuit[*i] == '(')
            total += ft_series(N, d, circuit, i);
        else if (circuit[*i] == '[')
            total += ft_parallel(N, d, circuit, i);
        else if (circuit[*i] != ' ')
        {
            total += (float)ft_dict_search(N, d, &circuit[*i]);
            while (circuit[*i] != ' ')
                *i += 1;
        }
        *i += 1;
    }
    //fprintf(stderr, "S %f\n", total);
    return (total);
}

int main()
{
    int N;
    scanf("%d", &N);
    t_dict  d[N];
    for (int i = 0; i < N; i++) {
        scanf("%s%d", d[i].name, &d[i].R); fgetc(stdin);
    }
    char circuit[121];
    scanf("%[^\n]", circuit);

    int     i;
    int     size;
    float   total;

    i = 0;
    size = strlen(circuit);
    total = 0;
    while (i < size)
    {
        if (circuit[i] == '(')
            total += ft_series(N, d, circuit, &i);
        else if (circuit[i] == '[')
            total += ft_parallel(N, d, circuit, &i);
        i++;
    }
    printf("%0.1f\n", total);

    return 0;
}
