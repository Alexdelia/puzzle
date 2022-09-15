unsigned short numberOfSteps(unsigned int n)
{
	unsigned short steps = 0;

	while (n > 0)
	{
		if (n % 2)
			n--;
		else
            n /= 2;
		steps++;
	}

	return (steps);
}