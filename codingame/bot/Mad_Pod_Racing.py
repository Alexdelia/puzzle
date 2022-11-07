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

def distance(x1: int, y1: int, x2: int, y2: int) -> float:
    return math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2)

class Pod:
    def __init__(self, x: int, y: int, vx: int, vy: int, angle: int, next_x: int, next_y: int):
        self.c: Tuple[int, int] = (x, y)
        self.s: Tuple[int, int] = (vx, vy)
        self.prev: Tuple[int, int] = (x - vx, y - vy)
        self.next_check: Tuple[int, int] = (next_x, next_y)
        self.dist: float = distance(x, y, next_x, next_y)
        self.angle: int = angle
        self.speed: float = distance(self.prev[0], self.prev[1], self.c[0], self.c[1])
        self.drift: float = -H_DRIFT * self.speed
    
    def update(self, x: int, y: int, vx: int, vy: int, angle: int, next_x: int, next_y: int):
        self.prev = self.c
        self.c = (x, y)
        self.s = (vx, vy)
        self.next_check = (next_x, next_y)
        self.dist = distance(x, y, next_x, next_y)
        self.angle = angle
        self.speed = distance(self.prev[0], self.prev[1], self.c[0], self.c[1])
        self.drift = -H_DRIFT * self.speed
    
    def get_thrust(self, opponent: List[Tuple[int, int]]) -> Union[int, str]:
        if self.angle > 90 or self.angle < -90:
            return 0
        elif abs(self.angle) < 3 and self.dist > 8000:
            for o in opponent:
                if distance(self.c[0], self.c[1], o[0], o[1]) > 2121:
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
        return (self.next_check[0] + int(self.drift * d_x), self.next_check[1] + int(self.drift * d_y))
        # return int(x), int(y)

    def __str__(self) -> str:
        ret = f"C: {self.c}\n"
        ret += f" P: {self.prev}\n"
        ret += f" N: {self.next_check}\n"
        ret += f" V: {self.s}\n"
        ret += f" D: {self.dist}\n"
        ret += f" A: {self.angle}\n"
        ret += f" Spd: {self.speed}\n"
        ret += f" Dft: {self.drift}"
        return ret


class Env:
    bot: List[Pod] = []
    opponent: List[Pod] = []
    n_laps: int
    n_check: int
    check: List[Tuple[int, int]] = []

    def init_info(self):
        self.n_laps = int(input())
        self.n_check = int(input())
        for _ in range(self.n_check):
            self.check.append(tuple([int(i) for i in input().split()]))

    def get_info(self, n_pod: int):
        for i in range(n_pod):
            x, y, vx, vy, angle, i_check = [int(i) for i in input().split()]
            if len(self.bot) < n_pod:
                self.bot.append(Pod(x, y, vx, vy, angle, *self.check[i_check]))
            else:
                self.bot[i].update(x, y, vx, vy, angle, *self.check[i_check])
        for i in range(n_pod):
            x, y, vx, vy, angle, i_check = [int(i) for i in input().split()]
            if len(self.opponent) < n_pod:
                self.opponent.append(Pod(x, y, vx, vy, angle, *self.check[i_check]))
            else:
                self.opponent[i].update(x, y, vx, vy, angle, *self.check[i_check])

    def __str__(self) -> str:
        ret = f"n_laps: {self.n_laps}\n"
        ret += f"n_check: {self.n_check}\n"
        ret += f"check: {self.check}\n"
        ret += "Bot:\n"
        for b in self.bot:
            ret += str(b) + "\n"
        ret += "Opponent:\n"
        for o in self.opponent:
            ret += str(o) + "\n"
        return ret

e = Env()
e.init_info()

# game loop
while True:
    e.get_info(2)
    print(e, file=sys.stderr)

    c_op = [o.c for o in e.opponent]
    for i, b in enumerate(e.bot):
        x, y = b.get_targeted_xy()
        thrust = b.get_thrust(c_op)
        print(f"{x} {y} {thrust} {thrust}")
