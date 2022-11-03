import math
import sys
from typing import Tuple, Union

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

T_ANGLE = 45
B_ANGLE = 90
T_ANGLE_SPEED = 1.0
B_ANGLE_SPEED = 0.5

T_DIST = 4000
B_DIST = 0
T_DIST_SPEED = 1.0
B_DIST_SPEED = 0.1

class Env:
    x: int
    y: int
    next_x: int     # x position of the next check point
    next_y: int     # y position of the next check point
    dist: int       # distance to the next checkpoint
    angle: int      # angle between your pod orientation and the direction of the next checkpoint
    opponent_x: int
    opponent_y: int


    def get_info(self):
        self.x, self.y, self.next_x, self.next_y, self.dist, self.angle = [int(i) for i in input().split()]
        self.opponent_x, self.opponent_y = [int(i) for i in input().split()]
    

    def get_thrust(self) -> Union[int, str]:
        if self.angle > 90 or self.angle < -90:
            return 0
        elif self.angle < 3 and self.dist > T_DIST \
                and abs(self.x - self.opponent_x) > 1500 and abs(self.y - self.opponent_y) > 1500:
            print(f"diff: {abs(self.x - self.opponent_x)} {abs(self.y - self.opponent_y)}", file=sys.stderr)
            return "BOOST"

        ### Angle
        # x          | 0   | B   | (B+T)/2 | T  | inf  (abs)
        # self.angle | 0   | 45  | 67.5    | 90 | 180  (abs)
        # s_angle    | 0.5 | 0.5 | 0.75    | 1  | 1
        if self.angle > B_ANGLE:
            s_angle = B_ANGLE_SPEED
        elif self.angle < T_ANGLE:
            s_angle = T_ANGLE_SPEED
        else:
            s_angle = B_ANGLE_SPEED + (T_ANGLE_SPEED - B_ANGLE_SPEED) * (self.angle - B_ANGLE) / (T_ANGLE - B_ANGLE)
        print(f"s_angle: {s_angle}", file=sys.stderr)
        
        ### Distance
        # x          | 0   | B   | (B+T)/2 | T    | inf  (abs)
        # self.dist  | 0   | 500 | 750     | 1000 | inf (abs)
        # s_dist     | 0   | 0   | 0.25    | 1    | 1
        if self.dist > T_DIST:
            s_dist = T_DIST_SPEED
        elif self.dist < B_DIST:
            s_dist = B_DIST_SPEED
        else:
            s_dist = B_DIST_SPEED + (T_DIST_SPEED - B_DIST_SPEED) * (self.dist - B_DIST) / (T_DIST - B_DIST)
        print(f"s_dist: {s_dist}", file=sys.stderr)

        return int(100 * (s_angle + s_dist) / 2)
    

    def get_targeted_xy(self) -> Tuple[int, int]:
        # stronger angle to get angle to angle == 0 faster
        if self.dist > 1000 or self.angle > 90 or self.angle < -90:
            return self.next_x, self.next_y
        angle = self.angle * 2
        x = self.x + math.cos(math.radians(angle)) * self.dist
        y = self.y + math.sin(math.radians(angle)) * self.dist
        return int(x), int(y)
    

    def debug(self):
        print(f"I: {self.x}\t{self.y}", file=sys.stderr)
        print(f"N: {self.next_x}\t{self.next_y}", file=sys.stderr)
        print(f"D: {self.dist}", file=sys.stderr)
        print(f"A: {self.angle}", file=sys.stderr)
        print(f"O: {self.opponent_x}\t{self.opponent_y}", file=sys.stderr)

e = Env()

# game loop
while True:
    e.get_info()
    e.debug()
    thrust = e.get_thrust()
    x, y = e.get_targeted_xy()
    print(f"{x} {y} {thrust}")
