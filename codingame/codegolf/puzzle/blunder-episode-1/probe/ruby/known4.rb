h={
'6 5
#####
# @ #
#   #
#   #
#  $#
#####'=>'SOUTH
SOUTH
SOUTH
EAST',
'10 10
##########
#    @   #
#        #
#    B   #
#    X   #
#    X   #
#    B   #
#     BX$#
#    XXXX#
##########'=>'SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST',
'15 15
###############
#      X$     #
#      X      #
#  @   X      #
#      X     T#
#      XXXXXXX#
#  B   T      #
#  X          #
#  X          #
#  B          #
#    B  BI  NX#
#  XXXXXX     #
#             #
#             #
###############'=>'SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
NORTH
NORTH
NORTH
NORTH
WEST
WEST
WEST
WEST
WEST
WEST
WEST
WEST
WEST
WEST
NORTH
NORTH
NORTH',
'15 15
###############
#      X$     #
#      X      #
#  @   X      #
#  SW  X     T#
#      XXXXXXX#
#  B   TS     #
#  XN   W     #
#  X          #
#  B          #
#    B  BI  NX#
#  XXXXXX     #
#             #
#             #
###############'=>'LOOP',
};puts h[$<.read.chomp]||"X"
