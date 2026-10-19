n = int(input())
edges = [input().split() for _ in range(n)]
print(len({y for _, y in edges}))
