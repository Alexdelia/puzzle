def is_flowing(words: list[str]) -> bool:
	return all(words[i][-1] == words[i + 1][0] for i in range(len(words) - 1))


print(str(is_flowing(input().split())).lower())
