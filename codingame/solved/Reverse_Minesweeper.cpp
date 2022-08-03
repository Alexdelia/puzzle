#include <iostream>
#include <string>
#include <vector>
#include <algorithm>

using namespace std;

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/

int main()
{
    int w;
    cin >> w; cin.ignore();
    int h;
    cin >> h; cin.ignore();

    int m[h + 2][w + 2];
    int x, y;

    for (x = 0; x < h + 2; x++)
        for (y = 0; y < w + 2; y++)
            m[x][y] = 0;

    for (x = 0; x < h; x++)
    {
        string line;
        getline(cin, line);
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
            cout << (m[x][y] > 0 ? (char)(m[x][y] + '0') : '.');
        cout << endl;
    }
}
