class Wordle
{
	answer

	constructor(answer)
	{
		this.answer = answer;
	}

	solve(attempt)
	{
		const flag = [false, false, false, false, false];
		const res = ['X', 'X', 'X', 'X', 'X'];

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

const answer = readline();

const wordle = new Wordle(answer);
var output = '';

const N = parseInt(readline());
for (let i = 0; i < N; i++)
{
	const attempt = readline();

	output += wordle.solve(attempt) + '\n';
}

console.log(output.trim());