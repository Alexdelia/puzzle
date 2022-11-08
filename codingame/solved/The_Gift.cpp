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
	vector<int> b;
    int n;
    cin >> n; cin.ignore();
    int c;
    cin >> c; cin.ignore();
    for (int i = 0; i < n; i++) {
        int t;
        cin >> t; cin.ignore();
		b.push_back(t);
    }

	vector<int> r = vector<int>(n, 0);
	size_t b_sum = 0;
	for (auto i : b)
		b_sum += i;
	size_t sum = 0;

	while (sum < b_sum)
	{
		for (size_t i = 0; i < b.size(); i++)
		{
			if (r[i] < b[i])
			{
				r[i]++;
				sum++;
			}
			if (sum == c)
			{
				sort(r.begin(), r.end());
				for (auto i : r)
					std::cout << i << std::endl;
				return 0;
			}
		}
	}

    // Write an answer using cout. DON'T FORGET THE "<< endl"
    // To debug: cerr << "Debug messages..." << endl;

    std::cout << "IMPOSSIBLE" << std::endl;
}