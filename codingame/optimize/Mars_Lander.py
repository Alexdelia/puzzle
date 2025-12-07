# Save the Planet.
# Use less Fossil Fuel.

land: list[tuple[int, int]] = []

n = int(input())  # the number of points used to draw the surface of Mars.
for i in range(n):
	# land_x: X coordinate of a surface point. (0 to 6999)
	# land_y: Y coordinate of a surface point. By linking all the points together in a sequential fashion, you form the surface of Mars.
	land.append(tuple(map(int, input().split())))

flat_x: tuple[int, int] = (0, 0)
flat_y: int = 0
# find coord to land
for i in range(len(land) - 1):
	if land[i][1] == land[i + 1][1]:
		flat_x = land[i][0], land[i + 1][0]
		flat_y = land[i][1]
		break

# game loop
while True:
	# hs: the horizontal speed (in m/s), can be negative.
	# vs: the vertical speed (in m/s), can be negative.
	# f: the quantity of remaining fuel in liters.
	# r: the rotation angle in degrees (-90 to 90).
	# p: the thrust power (0 to 4).
	x, y, hs, vs, f, r, p = [int(i) for i in input().split()]

	# factor of angle depending on the distance to flat from 0 to 1
	if x < flat_x[0]:
		dir_factor = -1
	elif x > flat_x[1]:
		dir_factor = -(1 - x / flat_x[1])
	else:
		dir_factor = 0

	# factor of angle depending on hs
	if y < flat_y + 100:
		hs_factor = 0
	elif hs >= 20:
		hs_factor = (hs - 20) / 10
	elif hs <= -20:
		hs_factor = (hs + 20) / 10
	else:
		hs_factor = 0

	# r depending on dir_factor and hs_factor
	r = int(90 * dir_factor + 90 * hs_factor)
	if r > 90:
		r = 90
	elif r < -90:
		r = -90

	# p depending on vs
	if vs < -38:
		p = 4
	elif vs < -20:
		p = 3
	elif vs < -10:
		p = 2
	elif vs < -5:
		p = 1
	else:
		p = 0

	# R P. R is the desired rotation angle. P is the desired thrust power.
	print(f"{int(r)} {int(p)}")
