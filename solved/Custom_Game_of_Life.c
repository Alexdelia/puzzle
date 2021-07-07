#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>

/*
**	Codingame Puzzle
*/

typedef struct  s_rules
{
    int         h;
    int         w;
    char        alive[10];
    char        dead[10];
}               t_r;

void    ft_print_grid(int h, int w, char grid[h][w], int out)
{
    int x;
    int y;

    /*if (out == 2)
        write(2, "\ngrid:\n", 7);*/
    if (out == 2)
        write(2, "\n", 1);
    x = 0;
    while (x < h)
    {
        y = 0;
        while (y < w)
        {
            write(out, &grid[x][y], 1);
            y++;
        }
        write(out, "\n", 1);
        x++;
    }
    if (out == 2)
        write(2, "\n", 1);
}

void    ft_cpy_grid(int h, int w, char dest[h][w], char src[h][w])
{
    int base_w;

    h--;
    w--;
    base_w = w;
    while (h >= 0)
    {
        w = base_w;
        while (w >= 0)
        {
            dest[h][w] = src[h][w];
            w--;
        }
        h--;
    }
}

int     ft_number_of_neighbour(int h, int w, char grid[h][w], int x, int y)
{
    int total;

    total = 0;

    //fprintf(stderr, "\n%d|%d %d|%d : ", h, w, x, y);

    h--;
    w--;

    if (x > 0)
    {
        if (y > 0 && grid[x - 1][y - 1] == 'O')
            total++;
        if (grid[x - 1][y] == 'O')
            total++;
        if (y < w&& grid[x - 1][y + 1] == 'O')
            total++;
    }

    if (y > 0 && grid[x][y - 1] == 'O')
        total++;
    if (y < w && grid[x][y + 1] == 'O')
        total++;

    if (x < h)
    {
        if (y > 0 && grid[x + 1][y - 1] == 'O')
            total++;
        if (grid[x + 1][y] == 'O')
            total++;
        if (y < w && grid[x + 1][y + 1] == 'O')
            total++;
    }

    return (total);
}

void    ft_GoL(t_r rules, char grid[rules.h][rules.w])
{
    char    base_grid[rules.h][rules.w];
    int     x;
    int     y;
    int     neighbour;

    ft_cpy_grid(rules.h, rules.w, base_grid, grid);
    //ft_print_grid(rules.h, rules.w, base_grid, 2);
    x = 0;
    while (x < rules.h)
    {
        y = 0;
        while (y < rules.w)
        {
            neighbour = ft_number_of_neighbour(rules.h, rules.w, base_grid, x, y);
            //fprintf(stderr, "%d%c", neighbour, (y == rules.w - 1 ? '\n' : ' '));
            if (base_grid[x][y] == 'O' && rules.alive[neighbour] == '0')
                grid[x][y] = '.';
            else if (base_grid[x][y] == '.' && rules.dead[neighbour] == '1')
                grid[x][y] = 'O';
            y++;
        }
        x++;
    }
    ft_print_grid(rules.h, rules.w, grid, 2);
}

int main()
{
    t_r rules;
    int n;
    scanf("%d%d%d", &rules.h, &rules.w, &n); fgetc(stdin);
    scanf("%[^\n]", rules.alive); fgetc(stdin);
    scanf("%[^\n]", rules.dead); fgetc(stdin);
    char grid[rules.h][rules.w];
    for (int i = 0; i < rules.h; i++) {
        scanf("%[^\n]", grid[i]); fgetc(stdin);
    }

    write(2, "Starting grid:\n", 15);
    ft_print_grid(rules.h, rules.w, grid, 2);

    while (n > 0)
    {
        ft_GoL(rules, grid);
        //ft_print_grid(rules.h, rules.w, grid, 2);
        n--;
    }

    ft_print_grid(rules.h, rules.w, grid, 1);

    return 0;
}
