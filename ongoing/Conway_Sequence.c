#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

void	ft_psequence(int *sequence)
{
	int	i;

	i = 0;
	while (sequence[i] != -1)
	{
		if (i > 0)
			printf(" ");
		printf("%d", sequence[i]);
		i++;
	}
	printf("\n");
}

int	ft_next_size(int *input)
{
	int	i;
	int	a;
	int	size;

	i = 0;
	size = 0;
	while (input[i] != -1)
	{
		a = input[i];
		while (input[i] != -1 && input[i] == a)
		{
			i++;
		}
		size += 2;
	}
	return (size);
}

int *ft_conway_sequence(int	*input)
{
	int	*output;
	int	o;
	int	i;
	int	a;
	int	ocu;

	if (!(output = malloc(sizeof(*output) * (ft_next_size(input) + 1))))
		return (NULL);
	i = 0;
	o = 0;
	while (input[i] != -1)
	{
		a = input[i];
		ocu = 0;
		while (input[i] != -1 && input[i] == a)
		{
			ocu++;
			i++;
		}
		output[o] = ocu;
		output[o + 1] = a;
		o += 2;
	}
	output[o] = -1;
	return (output);
}

int	ft_malloc_error(int	ret)
{
	fprintf(stderr, "Malloc error");
	return (ret);
}

int	*ft_find_L_sequence(int	R, int L)
{
	int	*sequence;
	int	*alt_sequence;
	int	i;

	if (!(sequence = malloc(sizeof(*sequence) * 3)))
		return (NULL);
	sequence[0] = R;
	sequence[1] = -1;
	i = 0;
	while (i < L)
	{
		if (!(alt_sequence = ft_conway_sequence(sequence)))
			return (NULL);
		free(sequence);
		i++;
		if (i >= L)
			return (alt_sequence);
		if (!(sequence = ft_conway_sequence(alt_sequence)))
			return (NULL);
		free(alt_sequence);
		i++;
	}
	return (sequence);
}

int main()
{
    int R;
    scanf("%d", &R);
    int L;
    scanf("%d", &L);

	int	*sequence;
	if (!(sequence = ft_find_L_sequence(R, L - 1)))
		return (ft_malloc_error(1));
	ft_psequence(sequence);
	free(sequence);
    return 0;
}
