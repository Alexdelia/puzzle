print(
    sum(x for x in (
        int(input()) for _ in range(int(input()))
    ) if x % 2 == 0)
)