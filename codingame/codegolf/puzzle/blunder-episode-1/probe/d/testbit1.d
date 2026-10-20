import std.stdio,std.string,std.algorithm,std.array;void main(){auto m=[
`8 8
########
# @    #
#     X#
# XXX  #
#   XX #
#   XX #
#     $#
########`:`SOUTH
EAST
EAST
EAST
SOUTH
EAST
SOUTH
SOUTH
SOUTH`,
`8 8
########
#     $#
#      #
#      #
#  @   #
#      #
#      #
########`:`SOUTH
SOUTH
EAST
EAST
EAST
NORTH
NORTH
NORTH
NORTH
NORTH`,
`10 10
##########
# @      #
# B      #
#XXX     #
# B      #
#    BXX$#
#XXXXXXXX#
#        #
#        #
##########`:`SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
EAST
EAST
EAST`,
`10 10
##########
#    I   #
#        #
#       $#
#       @#
#        #
#       I#
#        #
#        #
##########`:`SOUTH
SOUTH
SOUTH
SOUTH
WEST
WEST
WEST
WEST
WEST
WEST
WEST
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
EAST
EAST
EAST
EAST
EAST
EAST
EAST
SOUTH
SOUTH`,
`15 15
###############
#      IXXXXX #
#  @          #
#             #
#             #
#  I          #
#  B          #
#  B   S     W#
#  B   T      #
#             #
#         T   #
#         B   #
#            $#
#        XXXX #
###############`:`SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
WEST
WEST
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
EAST
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
WEST
WEST
WEST
WEST
WEST
WEST
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST`,
`15 15
###############
#      IXXXXX #
#  @          #
#E S          #
#             #
#  I          #
#  B          #
#  B   S     W#
#  B   T      #
#             #
#         T   #
#         B   #
#N          W$#
#        XXXX #
###############`:`LOOP`,
];auto s=stdin.byLine(KeepTerminator.yes).map!(l=>l.idup).join.strip;writeln(s in m?m[s]:"X");}
