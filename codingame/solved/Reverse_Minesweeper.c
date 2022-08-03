#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/

int main()
{
    int w;
    scanf("%d", &w);
    int h;
    scanf("%d", &h); fgetc(stdin);

    int m[h + 2][w + 2];
    int x, y;

    for (x = 0; x < h + 2; x++)
        for (y = 0; y < w + 2; y++)
            m[x][y] = 0;

    for (x = 0; x < h; x++) {
        char line[101];
        scanf("%[^\n]", line); fgetc(stdin);
        for (y = 0; y < w; y++)
            m[x + 1][y + 1] = (line[y] == '.' ? 0 : -10);
    }

    for (x = 1; x <= h; x++)
    {
        for (y = 1; y <= w; y++)
        {
            if (m[x][y] < 0)
            {
                m[x - 1][y - 1]++;
                m[x - 1][y]++;
                m[x - 1][y + 1]++;
                m[x][y - 1]++;
                m[x][y + 1]++;
                m[x + 1][y - 1]++;
                m[x + 1][y]++;
                m[x + 1][y + 1]++;
            }
        }
    }

    for (x = 1; x <= h; x++)
    {
        for (y = 1; y <= w; y++)
            putchar((m[x][y] > 0 ? (char)(m[x][y] + '0') : '.'));
        putchar('\n');
    }
}
