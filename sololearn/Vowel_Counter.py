VOWEL = frozenset("aeiou")

print(sum(1 for c in input().lower() if c in VOWEL))
