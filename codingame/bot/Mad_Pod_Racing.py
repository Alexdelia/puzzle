import math
import sys
from typing import List, Tuple, Union

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

T_ANGLE = 15
B_ANGLE = 90
T_ANGLE_SPEED = 1.0
B_ANGLE_SPEED = 0.0

T_DIST = 1200
B_DIST = 600
T_DIST_SPEED = 1.0
B_DIST_SPEED = 0.1

TILT = 1000

H_DRIFT = 3

class Pod:
    c: Tuple[int, int] = (-1, -1)
    prev: Tuple[int, int]
    dist: int
    angle: int
    drift: float
    speed: float

class Env:
    c: Tuple[int, int] = (-1, -1)
    prev: Tuple[int, int]
    checkpoint: Tuple[int, int]
    opponent: Tuple[int, int]
    dist: int       # distance to the next checkpoint
    angle: int      # angle between your pod orientation and the direction of the next checkpoint
    n_laps: int
    n_check: int
    check: List[Tuple[int, int]] = []
    drift: float = 0
    speed: float = 0

    def init_info(self):
        self.n_laps = int(input())
        self.n_check = int(input())
        for _ in range(self.n_check):
            self.check.append(tuple([int(i) for i in input().split()]))

    def get_info(self):
        x, y, next_x, next_y, self.dist, self.angle = [
            int(i) for i in input().split()]
        self.prev = [self.c, (x, y)][self.c == (-1, -1)]
        self.c = (x, y)
        self.speed = math.sqrt((self.c[0] - self.prev[0]) ** 2 +
                               (self.c[1] - self.prev[1]) ** 2)
        self.drift = -H_DRIFT * self.speed
        self.checkpoint = (next_x, next_y)
        if self.checkpoint not in self.check:
            self.check.append(self.checkpoint)
        op_x, op_y = [int(i) for i in input().split()]
        self.opponent = (op_x, op_y)

    def get_thrust(self) -> Union[int, str]:
        if self.angle > 90 or self.angle < -90:
            return 0
        elif abs(self.angle) < 3 and self.dist > 8000 \
                and abs(self.c[0] - self.opponent[0]) > 1500 and abs(self.c[1] - self.opponent[1]) > 1500:
            return "BOOST"

        # Angle
        # x          | 0   | B   | (B+T)/2 | T  | inf  (abs)
        # self.angle | 0   | 45  | 67.5    | 90 | 180  (abs)
        # s_angle    | 0.5 | 0.5 | 0.75    | 1  | 1
        if abs(self.angle) > B_ANGLE:
            s_angle = B_ANGLE_SPEED
        elif abs(self.angle) < T_ANGLE:
            s_angle = T_ANGLE_SPEED
        else:
            s_angle = B_ANGLE_SPEED + \
                (T_ANGLE_SPEED - B_ANGLE_SPEED) * \
                (abs(self.angle) - B_ANGLE) / (T_ANGLE - B_ANGLE)
        print(f"s_angle: {s_angle}", file=sys.stderr)

        # Distance
        # x          | 0   | B   | (B+T)/2 | T    | inf  (abs)
        # self.dist  | 0   | 500 | 750     | 1000 | inf (abs)
        # s_dist     | 0   | 0   | 0.25    | 1    | 1
        if self.dist > T_DIST:
            s_dist = T_DIST_SPEED
        elif self.dist < B_DIST:
            s_dist = B_DIST_SPEED
        else:
            s_dist = B_DIST_SPEED + \
                (T_DIST_SPEED - B_DIST_SPEED) * \
                (self.dist - B_DIST) / (T_DIST - B_DIST)
        print(f"s_dist: {s_dist}", file=sys.stderr)

        return int(100 * (s_angle + s_dist) / 2)

    def get_targeted_xy(self) -> Tuple[int, int]:
        # depending on drift, apply a correction offset to the target
        d_x = self.c[0] - self.prev[0]
        d_y = self.c[1] - self.prev[1]
        d = math.sqrt(d_x ** 2 + d_y ** 2)
        if d != 0:
            d_x /= d
            d_y /= d
        return (self.checkpoint[0] + int(self.drift * d_x), self.checkpoint[1] + int(self.drift * d_y))
        # return int(x), int(y)

    def init_debug(self):
        print(f"n_laps: {self.n_laps}", file=sys.stderr)
        print(f"n_check: {self.n_check}", file=sys.stderr)
        print(f"check: {self.check}", file=sys.stderr)

    def debug(self):
        print(f"I: {self.c}", file=sys.stderr)
        print(f"P: {self.prev}", file=sys.stderr)
        print(f"N: {self.checkpoint}", file=sys.stderr)
        print(f"O: {self.opponent}", file=sys.stderr)
        print(f"D: {self.dist}", file=sys.stderr)
        print(f"A: {self.angle}", file=sys.stderr)
        print(f"Dft: {self.drift}", file=sys.stderr)
        print(f"Spd: {self.speed}", file=sys.stderr)


e = Env()
e.init_info()

# game loop
while True:
    e.get_info()
    e.debug()
    thrust = e.get_thrust()
    x, y = e.get_targeted_xy()
    print(f"{x} {y} {thrust} {thrust}")
