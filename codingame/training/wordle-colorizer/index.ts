
type Result = '#' | 'O' | 'X';

class Wordle
{
	private readonly answer: string;

	constructor(answer: string)
	{
		this.answer = answer;
	}

	public solve(attempt: string): string
	{
		const flag = [false, false, false, false, false];
		const res: Result[] = ['X', 'X', 'X', 'X', 'X'];

		for (let i = 0; i < 5; i++)
		{
			if (attempt[i] === this.answer[i])
			{
				flag[i] = true;
				res[i] = '#';
			}
		}

		for (let x = 0; x < 5; x++)
		{
			if (res[x] !== 'X')
			{
				continue;
			}

			for (let y = 0; y < 5; y++)
			{
				if (flag[y])
				{
					continue;
				}

				if (attempt[x] === this.answer[y])
				{
					flag[y] = true;
					res[x] = 'O';
					break;
				}
			}
		}

		console.error(attempt);
		console.error(flag.map((v) => v ? 1 : 0).join(''));
		console.error(res.join(''));
		console.error('');

		return res.join('');
	}
}

const answer: string = readline();

const wordle: Wordle = new Wordle(answer);
var output: string = '';

const N: number = parseInt(readline());
for (let i = 0; i < N; i++)
{
	const attempt: string = readline();

	output += wordle.solve(attempt) + '\n';
}

console.log(output.trim());