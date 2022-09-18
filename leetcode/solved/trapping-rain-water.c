unsigned int trap(unsigned int *h, unsigned int heightSize)
{
	/*if (heightSize < 3)
		return (0);*/	// not needed

	unsigned int	*l;
	unsigned int	*r;
	unsigned int	max_l = 0;
	unsigned int	max_r = 0;
	unsigned int	water = 0;

	l = &h[0];
	r = &h[heightSize - 1];

	while (l != r + 1)
	{
		if (max_l < max_r)
		{
			if (*l > max_l)
				max_l = *l;
			else
				water += max_l - *l;
			++l;
		}
		else
		{
			if (*r > max_r)
				max_r = *r;
			else
				water += max_r - *r;
			--r;
		}
	}

	return (water);
}
