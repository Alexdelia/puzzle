import 'dart:io';void main(){var m={
r'''10 10
##########
#    T   #
#        #
#        #
#        #
#@       #
#        #
#        #
#    T  $#
##########''':r'''SOUTH
SOUTH
SOUTH
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
SOUTH''',
r'''10 10
##########
#        #
#  @     #
#  B     #
#  S   W #
# XXX    #
#  B   N #
# XXXXXXX#
#       $#
##########''':r'''SOUTH
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
EAST''',
r'''15 15
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
###############''':r'''SOUTH
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
EAST''',
r'''15 15
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
###############''':r'''LOOP''',
r'''30 15
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
###############''':r'''SOUTH
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
EAST''',
};var l=<String>[];String? x;while((x=stdin.readLineSync())!=null)l.add(x!);var s=l.join('\n').trim();print(m[s]??'X');}
