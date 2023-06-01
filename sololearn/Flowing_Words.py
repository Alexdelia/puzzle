def is_flowing(words: list[str]) -> bool:
    for i in range(len(words) - 1):
        if words[i][-1] != words[i + 1][0]:
            return False
    return True

print(str(
    is_flowing(input().split())
).lower())