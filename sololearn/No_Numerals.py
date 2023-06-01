o = []

for w in input().split():
    if (len(w) == 1 and w.isdigit()) or w == "10":
        o.append([
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten"
        ][int(w)])
    else:
        o.append(w)

print(*o)