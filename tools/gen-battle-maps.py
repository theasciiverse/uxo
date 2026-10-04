#!/usr/bin/env python3
# One-off: laid out the first versions of the maps. The .txt files are the source of truth now;
# rerunning this overwrites hand edits. Usage: python3 tools/gen-battle-maps.py stages
"""Generate uXo's 25x17 battle maps once; the .txt files are the source of truth after."""
import random, sys

W, H = 25, 17
OUT = sys.argv[1]


def base(pillars=True):
    g = [['?' for _ in range(W)] for _ in range(H)]
    for y in range(H):
        for x in range(W):
            if x in (0, W - 1) or y in (0, H - 1):
                g[y][x] = '#'
            elif pillars and x % 2 == 0 and y % 2 == 0:
                g[y][x] = '#'
    return g


def starts(g):
    for (x, y, c) in [(1, 1, '1'), (W - 2, H - 2, '2'), (W - 2, 1, '3'), (1, H - 2, '4')]:
        g[y][x] = c
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1), (2, 0), (-2, 0), (0, 2), (0, -2)]:
            nx, ny = x + dx, y + dy
            if 0 < nx < W - 1 and 0 < ny < H - 1 and g[ny][nx] in '?#' and abs(dx) + abs(dy) == 1:
                g[ny][nx] = '.'
    return g


def put(g, x, y, c):
    g[y][x] = c


def line(g, x0, y0, x1, y1, c):
    for y in range(min(y0, y1), max(y0, y1) + 1):
        for x in range(min(x0, x1), max(x0, x1) + 1):
            g[y][x] = c


def write(name, about, g, extra=''):
    fn = f"{OUT}/{name}"
    body = '\n'.join(''.join(r) for r in g)
    with open(fn, 'w') as f:
        f.write(f"name: {about[0]}\nabout: {about[1]}\n{extra}grid:\n{body}\n")
    print(fn)
    print(body)
    print()


# 1. Open Field: the classic, just bigger
g = starts(base())
write('01-open-field.txt', ('Open Field', 'the classic arena, stretched out'), g)

# 2. Conveyor Works: an outer belt loop clockwise, an inner loop the other way round a
# machine room in the middle
g = base()
for x in range(3, 21):
    put(g, x, 3, '>')
for y in range(3, 13):
    put(g, 21, y, 'v')
for x in range(4, 22):
    put(g, x, 13, '<')
for y in range(4, 14):
    put(g, 3, y, '^')
put(g, 3, 3, '>')
put(g, 21, 13, '<')
# inner loop, anticlockwise
for x in range(10, 15):
    put(g, x, 6, '<')
for y in range(6, 10):
    put(g, 9, y, 'v')
for x in range(9, 15):
    put(g, x, 10, '>')
for y in range(7, 11):
    put(g, 15, y, '^')
put(g, 9, 6, 'v')
put(g, 15, 10, '^')
put(g, 15, 6, '<')
put(g, 9, 10, '>')
# the machine: solid, with a floor apron
line(g, 11, 7, 13, 9, '#')
line(g, 10, 7, 10, 9, '.')
line(g, 14, 7, 14, 9, '.')
line(g, 11, 6, 13, 6, '<')
# belts never sit under a pillar; open floor beside the outer loop's corners
for (x, y) in [(2, 3), (3, 2), (22, 3), (21, 2), (2, 13), (3, 14), (22, 13), (21, 14)]:
    if g[y][x] == '#':
        g[y][x] = '.'
g = starts(g)
write('02-conveyor-works.txt', ('Conveyor Works', 'a factory floor; the belt loops carry players and bombs'), g)

# 3. Ricochet: a racetrack round a walled court; arrows send kicked bombs round it
g = base()
line(g, 3, 3, 21, 3, '.')
line(g, 3, 13, 21, 13, '.')
line(g, 3, 3, 3, 13, '.')
line(g, 21, 3, 21, 13, '.')
put(g, 3, 3, 'R')
put(g, 21, 3, 'D')
put(g, 21, 13, 'L')
put(g, 3, 13, 'U')
# the court wall with four gates
line(g, 5, 5, 19, 5, '#')
line(g, 5, 11, 19, 11, '#')
line(g, 5, 5, 5, 11, '#')
line(g, 19, 5, 19, 11, '#')
for (x, y) in [(12, 5), (12, 11), (5, 8), (19, 8)]:
    put(g, x, y, '.')
# no pillar may block a gate from the track side
for (x, y) in [(12, 4), (12, 12), (4, 8), (20, 8)]:
    put(g, x, y, '.')
# inside the court: a cross of arrows sends bombs out through the gates
put(g, 12, 8, '.')
line(g, 7, 8, 17, 8, '.')
line(g, 12, 6, 12, 10, '.')
g = starts(g)
write('03-ricochet.txt', ('Ricochet', 'a racetrack round a walled court; arrows send kicked bombs round it'), g, 'give: kick\n')

# 4. Warp Garden: four walled garden quarters, a warp in each, gates between
g = base()
line(g, 12, 1, 12, 15, '#')
line(g, 1, 8, 23, 8, '#')
for (x, y) in [(12, 3), (12, 13), (5, 8), (19, 8)]:
    put(g, x, y, '.')
for (x, y) in [(6, 4), (18, 4), (6, 12), (18, 12)]:
    put(g, x, y, 'o')
    for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
        if g[y + dy][x + dx] == '#' and 0 < x + dx < W - 1 and 0 < y + dy < H - 1 and (x + dx) != 12 and (y + dy) != 8:
            g[y + dy][x + dx] = '.'
# little ponds of open lawn in each quarter
for (x, y) in [(3, 5), (21, 5), (3, 11), (21, 11)]:
    put(g, x, y, '.')
g = starts(g)
write('04-warp-garden.txt', ('Warp Garden', 'four walled garden quarters, linked by warps'), g)

# 5. Ice Rink: an oval rink behind the boards, bumpers on the ice
g = base()
for y in range(3, 14):
    for x in range(5, 20):
        g[y][x] = '~'
for (x, y) in [(5, 3), (19, 3), (5, 13), (19, 13), (6, 3), (18, 3), (5, 4), (19, 4), (6, 13), (18, 13), (5, 12), (19, 12)]:
    g[y][x] = '#'
line(g, 4, 2, 20, 2, '#')
line(g, 4, 14, 20, 14, '#')
line(g, 4, 2, 4, 14, '#')
line(g, 20, 2, 20, 14, '#')
for (x, y) in [(12, 2), (12, 14), (4, 8), (20, 8)]:
    put(g, x, y, '.')
for (x, y) in [(9, 6), (15, 6), (9, 10), (15, 10), (12, 8)]:
    put(g, x, y, '#')
g = starts(g)
write('05-ice-rink.txt', ('Ice Rink', 'an oval rink behind the boards; on ice you slide until something stops you'), g)

# 6. Catacombs: a maze of corridors with a few rooms, lit
rnd = random.Random(1997)
g = [['#' for _ in range(W)] for _ in range(H)]
cells = [(x, y) for y in range(1, H, 2) for x in range(1, W, 2)]
seen = {(1, 1)}
stack = [(1, 1)]
g[1][1] = '.'
while stack:
    x, y = stack[-1]
    nb = [(x + dx, y + dy, dx, dy) for dx, dy in [(2, 0), (-2, 0), (0, 2), (0, -2)] if 0 < x + dx < W - 1 and 0 < y + dy < H - 1 and (x + dx, y + dy) not in seen]
    if not nb:
        stack.pop()
        continue
    nx, ny, dx, dy = rnd.choice(nb)
    g[y + dy // 2][x + dx // 2] = '.'
    g[ny][nx] = '.'
    seen.add((nx, ny))
    stack.append((nx, ny))
# knock through extra walls so there are loops to run round
for _ in range(34):
    x, y = rnd.randrange(1, W - 1), rnd.randrange(1, H - 1)
    if g[y][x] == '#' and (x % 2) != (y % 2):
        g[y][x] = '.'
# three crypt rooms
for (x0, y0) in [(10, 6), (3, 11), (17, 3)]:
    line(g, x0, y0, x0 + 4, y0 + 2, '.')
# soft blocks in the corridors
for y in range(1, H - 1):
    for x in range(1, W - 1):
        if g[y][x] == '.' and rnd.random() < 0.55:
            g[y][x] = '?'
g = starts(g)
write('06-catacombs.txt', ('Catacombs', 'a maze of crypt corridors and three burial rooms'), g)
