"""
def calc_profit(sold: int) -> int:
    return sold * 3 - 21

def profit_to_str(p: int) -> str:
    if p > 0:
        return "Profit"
    elif p < 0:
        return "Loss"
    else:
        return "Broke Even"

print(profit_to_str(calc_profit(int(input()))))
"""

n = int(input())

if n > 7:
	print("Profit")
elif n < 7:
	print("Loss")
else:
	print("Broke Even")
