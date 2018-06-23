#!/usr/bin/env python

from random import randint, seed

def genbench(fname, cap, n):
    with open(fname, 'w') as fp:
        fp.write(str(cap) + '\n')
        for _ in range(n):
            k = randint(0, cap-1)
            v = randint(0, 1000)
            fp.write("{} {}\n".format(k, v))

if __name__ == '__main__':
    seed(4)
    genbench('bench_1', 1000, 10000)
