bool canConstruct(const char *ransomNote, const char *magazine)
{
	short			c[26] = {0};
	unsigned short	i;

	for (i = 0; ransomNote[i]; i++)
		c[ransomNote[i] - 'a']--;
	for (i = 0; magazine[i]; i++)
		c[magazine[i] - 'a']++;

	for (i = 0; i < 26; i++)
		if (c[i] < 0)
			return (false);
	return (true);
}