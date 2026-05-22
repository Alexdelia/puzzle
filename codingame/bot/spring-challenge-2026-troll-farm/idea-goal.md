1. train an harvester
stats:
- move speed 2
- carry capacity 2
- harvest 2
- chop 0

to get there, need enough fruit to train
so the first troll will pick from the tent and plant close the the tent
must have 2 plum and 2 lemon in a range of 3 cell from the tent, planting closesty to tent first (tie breaker by closest to troll) (already present tree count)

2. train carrier
stats:
- move speed 3
- carry capacity 4
- harvest 1
- chop 2

to get there, need enough fruit to train
first priority is harvesting tree (sum tree dist to troll + tree dist to tent) to plant at 4 lemon and 4 plum with same rule as goal 1.
second priority:
- if apple are needed, harvester troll (2, 2, 2, 0) will harvest from closest apple tree
- if iron are needed, carrier troll (3, 4, 1, 2) will chop from closest iron
third priority harvest to store in tent (must optimize best candidate to harvest depending on distance and carry capacity to tree capacity)

3. train woodcutter
stats:
- move speed 2
- carry capacity 4
- harvest 0
- chop 3

to get there, need enough fruit to train
first priority:
- if apple are needed, harvester troll (2, 2, 2, 0) will harvest from closest apple tree
- if iron are needed, carrier troll (3, 4, 1, 2) will chop from closest iron
second priority is harvesting tree best tree candidate

4. gather point

initial troll (1, 1, 1, 1) and harvester troll (2, 2, 2, ) will harvest closest banana tree to itself, to plant to closest empty space
(if no banana tree, pick from tent, if no banana in tent try any other tree without caring about the fruit type)
other troll will chop best tree candidate

switch to endgame when turn >= 280

5. endgame

harvester troll (2, 2, 2, 0) will harvest and plant banana tree just like in goal 4.
all other troll best tree candidate closest to tent
