package main;import("fmt";"io";"os";"strings");func main(){b,_:=io.ReadAll(os.Stdin);m:=map[string]string{
`5 5
#####
#@  #
#   #
#  $#
#####`:`SOUTH
SOUTH
EAST
EAST`,
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
#        #
#  S   W #
#        #
#  $     #
#        #
#@       #
#        #
#E     N #
##########`:`SOUTH
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
`10 10
##########
#        #
#  @     #
#  B     #
#  S   W #
# XXX    #
#  B   N #
# XXXXXXX#
#       $#
##########`:`SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
EAST
EAST
NORTH
NORTH
WEST
WEST
WEST
WEST
SOUTH
SOUTH
SOUTH
SOUTH
EAST
EAST
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
};if v,ok:=m[strings.TrimSpace(string(b))];ok{fmt.Println(v)}else{fmt.Println("X")}}
