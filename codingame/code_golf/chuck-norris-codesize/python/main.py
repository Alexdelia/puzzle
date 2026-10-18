import re
print(*[f"{'0'*(g<'1')}0 {'0'*len(g)}"for g in re.findall('0+|1+',''.join(f'{ord(c):07b}'for c in input()))])
