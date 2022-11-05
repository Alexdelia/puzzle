import math
import sys
from typing import List, Tuple, Union

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.


class Zone:
    def __init__(self, x: int, y: int, owner: int):
        self.x = x
        self.y = y
        self.owner = owner
        self.cost = 0
        self.to_beat = 0
        self.d: List[int] = []
    
    def __repr__(self):
        return f"\t({self.x}, {self.y})\n\tO: {self.owner}\n\tC: {self.cost}\n\tto_beat:{self.to_beat}"


class Drone:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y
        self.target: Tuple[int, int] = (x, y)
    
    def __repr__(self):
        return f"\t({self.x}, {self.y})\n\tT: {self.target}"


class Env:
    def __init__(self):
        # n_p: number of players in the game (2 to 4 players)
        # _id: ID of your player (0, 1, 2, or 3)
        # n_d: number of drones in each team (3 to 11)
        # n_z: number of zones on the map (4 to 8)
        self.n_p: int
        self._id: int
        self.n_d: int
        self.n_z: int
        self.d: List[List[Drone]] = [[], [], [], []]
        self.z: List[Zone] = [] # will be min heap
        self.free_d: List[int] = []

    def init_info(self):
        self.n_p, self._id, self.n_d, self.n_z = [
            int(i) for i in input().split()]
        for _ in range(self.n_z):
            # x: corresponds to the position of the center of a zone. A zone is a circle with a radius of 100 units.
            x, y = [int(j) for j in input().split()]
            self.z.append(Zone(x, y, -1))

    def get_info(self):
        for i in range(self.n_z):
            # ID of the team controlling the zone (0, 1, 2, or 3) or -1 if it is not controlled.
            tid = int(input())
            self.z[i].owner = tid

        if self.d == [[], [], [], []]:
            for pid in range(self.n_p):
                for _ in range(self.n_d):
                    x, y = [int(i) for i in input().split()]
                    self.d[pid].append(Drone(x, y))
        else:
            for pid in range(self.n_p):
                for did in range(self.n_d):
                    x, y = [int(i) for i in input().split()]
                    self.d[pid][did].x = x
                    self.d[pid][did].y = y
    
    def debug(self):
        print("Zones:", file=sys.stderr)
        for zid, z in enumerate(self.z):
            print(f"{zid}: {z}")
        print("Drones:", file=sys.stderr)
        for pid in range(self.n_p):
            print(f"Player {pid}:", file=sys.stderr)
            for did, d in enumerate(self.d[pid]):
                print(f"\t{did}: {d}", file=sys.stderr)
    
    @staticmethod
    def get_distance(x1: int, y1: int, x2: int, y2: int) -> float:
        return math.sqrt((x1 - x2) ** 2 + (y1 - y2) ** 2)
    
    @staticmethod
    def is_d_in_z(d: Drone, z: Zone) -> bool:
        return Env.get_distance(d.x, d.y, z.x, z.y) <= 100
    
    @staticmethod
    def is_c_in_c_100(x1: int, y1: int, x2: int, y2: int) -> bool:
        return Env.get_distance(x1, y1, x2, y2) <= 100

    def get_nearest_zid(self, x: int, y: int) -> int:
        min_dist = sys.maxsize
        min_id = -1
        for zid, z in enumerate(self.z):
            dist = self.get_distance(x, y, z.x, z.y)
            if dist < min_dist:
                min_dist = dist
                min_id = zid
        return min_id

    def get_nearest_did(self, x: int, y: int, free: bool = False) -> int:
        min_dist = sys.maxsize
        min_id = -1
        if free:
            d = self.free_d
        else:
            d = list(range(self.n_d))
        for did in d:
            dist = self.get_distance(x, y, self.d[self._id][did].x, self.d[self._id][did].y)
            if dist < min_dist:
                min_dist = dist
                min_id = did
        return min_id

    def update_to_beat(self):
        for z in self.z:
            z.to_beat = 0
        for pid in range(self.n_p):
            if pid == self._id:
                continue
            for d in self.d[pid]:
                for z in self.z:
                    if self.is_d_in_z(d, z):
                        z.to_beat += 1
                        break
    
    def update_d_in_z(self):
        for z in self.z:
            z.d = []
        for did, d in enumerate(self.d[self._id]):
            for z in self.z:
                if self.is_c_in_c_100(d.target[0], d.target[1], z.x, z.y):
                    z.d.append(did)
                    break
    
    def update_free_d(self):
        self.free_d = []
        for did, d in enumerate(self.d[self._id]):
            if d.target[0] == d.x and d.target[1] == d.y:
                self.free_d.append(did)
        # remove drones that are in a owned zone until z.d == z.to_beat + 1
        for z in self.z:
            if z.owner != self._id:
                continue
            for did in self.free_d:
                if did in z.d and len(z.d) > z.to_beat + 1:
                    self.free_d.remove(did)
                    z.d.remove(did)
    
    def update_cost(self):
        for z in self.z:
            z.cost = z.to_beat - len(z.d) + 1

    def update_target(self):
        queue = self.create_queue()
        for l in queue:
            for zid in l:
                # doesnt' involve finding best match between free_d and all zid from queue[cost]
                did = self.get_nearest_did(self.z[zid].x, self.z[zid].y, True)
                if did == -1:
                    return
                self.d[self._id][did].target = (self.z[zid].x, self.z[zid].y)
                self.free_d.remove(did)

    def create_queue(self) -> List[List[int]]:
        queue: List[List[int]] = [[] for _ in range(self.n_d + 1)]

        for zid, z in enumerate(self.z):
            if z.cost <= 0 or z.owner == self._id:
                continue
            queue[z.cost].append(zid)

        return queue

e = Env()
e.init_info()

# game loop
while True:
    e.get_info()

    e.update_to_beat()
    e.update_d_in_z()
    e.update_free_d()
    e.update_cost()

    e.update_target()

    # e.debug()

    for did in range(e.n_d):
        print(f"{e.d[e._id][did].target[0]} {e.d[e._id][did].target[1]}")
