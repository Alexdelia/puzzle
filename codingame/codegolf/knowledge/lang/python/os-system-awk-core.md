# os.system with the awk core

`import os;os.system("sort -nr|awk '$1+$2>e&&e?0:$2&&e=$1'|wc -l")` solves super-computer in 65 (verified, CG 100 %). Pattern: when stdin is a text stream and a shell/awk pipeline is shorter than native python. `from os import*;system(` costs the same as `import os;os.system(`.
