#include <iostream>
#include <ostream>
#include <string>
#include <map>
#include <cctype>
#include <algorithm>

/*
**	Codingame Puzzle
*/

#define DEBUG   0

std::map<std::string, std::string>  m;

const std::string    extension(const std::string &FNAME)
{
	std::string ret = FNAME.substr(FNAME.find_last_of('.') + 1);

	if (DEBUG)
		std::cerr << "EXT: " << ret << "\t" << FNAME << std::endl;

	if (ret == FNAME)
		return (std::string());
	std::transform(ret.begin(), ret.end(), ret.begin(),
			[](unsigned char c){ return std::tolower(c); });
	return (ret); 
}

const std::string   mime(const std::string &FNAME)
{
	const std::string   EXT = extension(FNAME);

	if (m.find(EXT) != m.end())
		return (m[EXT]);
	return ("UNKNOWN");
}

int main()
{
	int N; // Number of elements which make up the association table.
	std::cin >> N; std::cin.ignore();
	int Q; // Number Q of file names to be analyzed.
	std::cin >> Q; std::cin.ignore();

	for (int i = 0; i < N; i++)
	{
		std::string  EXT; // file extension
		std::string  MT; // MIME type.
		std::cin >> EXT >> MT; std::cin.ignore();

		if (DEBUG)
			std::cerr << EXT;

		// filling my dictionary
		std::transform(EXT.begin(), EXT.end(), EXT.begin(),
				[](unsigned char c){ return std::tolower(c); });

		if (DEBUG)
			std::cerr << " | " << EXT << "\t: " << MT << std::endl;

		m[EXT] = MT;
	}

	if (DEBUG)
		std::cerr << std::endl;

	for (int i = 0; i < Q; i++)
	{
		std::string FNAME;
		getline(std::cin, FNAME); // One file name per line.

		const std::string   MIME = mime(FNAME);

		if (DEBUG)
			std::cerr << FNAME << "\t" << MIME << std::endl;

		std::cout << MIME << std::endl;
	}

	return (0);
}
