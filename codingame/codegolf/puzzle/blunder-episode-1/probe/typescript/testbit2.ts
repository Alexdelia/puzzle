const m:{[k:string]:string}={
[`8 8
########
#      #
# @    #
# XX   #
#  XX  #
#   XX #
#     $#
########`]:`EAST
EAST
EAST
EAST
SOUTH
SOUTH
SOUTH
SOUTH`,
[`10 10
##########
#        #
#  S   W #
#        #
#  $     #
#        #
#@       #
#        #
#E     N #
##########`]:`SOUTH
SOUTH
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
NORTH
NORTH
WEST
WEST
WEST
WEST
SOUTH
SOUTH`,
[`10 10
##########
# @      #
# B      #
#XXX     #
# B      #
#    BXX$#
#XXXXXXXX#
#        #
#        #
##########`]:`SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
EAST
EAST
EAST`,
[`10 10
##########
#    I   #
#        #
#       $#
#       @#
#        #
#       I#
#        #
#        #
##########`]:`SOUTH
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
[`30 15
###############
#  #@#I  T$#  #
#  #    IB #  #
#  #     W #  #
#  #      ##  #
#  #B XBN# #  #
#  ##      #  #
#  #       #  #
#  #     W #  #
#  #      ##  #
#  #B XBN# #  #
#  ##      #  #
#  #       #  #
#  #     W #  #
#  #      ##  #
#  #B XBN# #  #
#  ##      #  #
#  #       #  #
#  #       #  #
#  #      ##  #
#  #  XBIT #  #
#  #########  #
#             #
# ##### ##### #
# #     #     #
# #     #  ## #
# #     #   # #
# ##### ##### #
#             #
###############`]:`SOUTH
SOUTH
SOUTH
SOUTH
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
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
NORTH
WEST
WEST
WEST
WEST
WEST
SOUTH
SOUTH
EAST
EAST
EAST
EAST
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
NORTH
WEST
WEST
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
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
NORTH
WEST
WEST
WEST
WEST
WEST
SOUTH
SOUTH
EAST
EAST
EAST
EAST
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
NORTH
NORTH
WEST
WEST
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
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
SOUTH
SOUTH
SOUTH
WEST
WEST
WEST
WEST
WEST
SOUTH
SOUTH
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
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
EAST`,
};const s=require("fs").readFileSync(0,"utf8").trim();console.log(m[s]??"X");
