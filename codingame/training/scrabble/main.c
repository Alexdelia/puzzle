#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

typedef struct s_dict
{
    char    w[31];
}           t_dict;

int ft_process_score(char word[31], int size)
{
                        //   a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q ,r,s,t,u,v,x,y,z
    const int   score[27] = {1,3,3,2,1,4,2,4,1,8,5,1,3,1,1,3,10,1,1,1,1,4,8,4,10};
    int         total;
    int         i;

    i = 0;
    total = 0;
    while (i < size)
    {
        total += score[word[i] - 'a'];
        i++;
    }
    return (total);
}

int ft_score(t_dict d, char l[8])
{
    int used[8] = { 0 };
    int i;
    int li;
    int size;

    i = 0;
    size = strlen(d.w);
    while (i < size)
    {
        li = 0;
        while (li < 8)
        {
            if (d.w[i] == l[li] && used[li] == 0)
            {
                used[li] = 1;
                li = 9;
            }
            li++;
        }
        if (li == 8)
            return (-1);
        i++;
    }
    return (ft_process_score(d.w, size));
}

int main()
{
    int N;
    scanf("%d", &N); fgetc(stdin);
    t_dict  d[N];
    for (int i = 0; i < N; i++) {
        scanf("%[^\n]", d[i].w); fgetc(stdin);
    }
    char l[8];
    scanf("%[^\n]", l);

    int i;
    int index;
    int score;
    int tmp;

    i = 0;
    index = 0;
    score = -1;
    while (i < N)
    {
        tmp = ft_score(d[i], l);
        if (tmp > score)
        {
            score = tmp;
            index = i;
        }
        i++;
    }
    printf("%s\n", d[index].w);

    return 0;
}
