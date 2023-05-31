def pig_latin(s: str) -> str:
    return s[1:] + s[0] + "ay"

print(*(pig_latin(w) for w in input().split()))