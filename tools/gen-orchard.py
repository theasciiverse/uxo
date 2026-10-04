#!/usr/bin/env python3
# One-off: laid out the first versions of the Orchard stages. The .txt files are the source of
# truth now; rerunning this overwrites hand edits. Usage: python3 tools/gen-orchard.py adventure/orchard
"""Lay out the Orchard's nine stages once; the .txt files are the source of truth after."""
import random, sys

OUT = sys.argv[1]
LEGEND = "baddie: g grub\nbaddie: b beetle\nbaddie: h hedgehog\nbaddie: f frog\nbaddie: r ram\nbaddie: c crow\nbaddie: x bramblehog\n"


def base(w, h, pillars=True, fill='?'):
    g = [[fill for _ in range(w)] for _ in range(h)]
    for y in range(h):
        for x in range(w):
            if x in (0, w - 1) or y in (0, h - 1):
                g[y][x] = '#'
            elif pillars and x % 2 == 0 and y % 2 == 0:
                g[y][x] = '#'
    return g


def line(g, x0, y0, x1, y1, c):
    for y in range(min(y0, y1), max(y0, y1) + 1):
        for x in range(min(x0, x1), max(x0, x1) + 1):
            g[y][x] = c


def clear_round(g, x, y, r=1):
    for yy in range(y - r, y + r + 1):
        for xx in range(x - r, x + r + 1):
            if 0 < xx < len(g[0]) - 1 and 0 < yy < len(g) - 1 and g[yy][xx] == '?':
                g[yy][xx] = '.'


def finish(g, exit_at, baddies, duds=()):
    w, h = len(g[0]), len(g)
    # the two starts, top left, with room to dodge
    for (x, y) in [(1, 1), (2, 1), (3, 1), (1, 2), (1, 3)]:
        g[y][x] = '.'
    g[1][1] = '1'
    g[1][3] = '2'
    ex, ey = exit_at
    g[ey][ex] = 'E'
    clear_round(g, ex, ey)
    for (x, y, c) in baddies:
        g[y][x] = c
        # somewhere to walk
        for dx, dy in [(1, 0), (-1, 0)]:
            if g[y + dy][x + dx] == '?':
                g[y + dy][x + dx] = '.'
    for (x, y) in duds:
        g[y][x] = '*'
    return g


def write(fname, name, story, goal, time, g, extra=''):
    body = '\n'.join(''.join(r) for r in g)
    st = ''.join(f"story: {l}\n" for l in story)
    with open(f"{OUT}/{fname}", 'w') as f:
        f.write(f"name: {name}\nworld: The Orchard\ntheme: orchard\ngoal: {goal}\ntime: {time}\n{st}{extra}{LEGEND}grid:\n{body}\n")
    print(fname, f"{len(g[0])}x{len(g)}")
    print(body)
    print()


rnd = random.Random(1984)

# 1-1 Windfall Lane: the classic pattern, a few grubs
g = base(19, 13)
g = finish(g, (17, 11), [(9, 5, 'g'), (13, 3, 'g'), (5, 9, 'g'), (15, 9, 'g')])
write('01-windfall-lane.txt', 'Windfall Lane', [
    "The night before the Festival of Lights, the workshop went quiet.",
    "Every firework you made for it is gone, scattered over the hills,",
    "come down as duds all through the Orchard. And the Orchard has",
    "gone strange. Bring the festival back, one stage at a time.",
    "Here: clear the lane of grubs, and the hatch opens.",
], 'clear', 150, g)

# 1-2 Beetle Rows: tree rows (hedge lines) with gaps
g = base(21, 13, pillars=False, fill='.')
for y in (3, 5, 7, 9):
    for x in range(2, 19):
        g[y][x] = '#' if x % 4 == 2 else '?'
    for x in (rnd.randrange(3, 9), rnd.randrange(11, 18)):
        g[y][x] = '.'
for y in (2, 4, 6, 8, 10, 11):
    for x in range(1, 20):
        if g[y][x] == '.' and rnd.random() < 0.3:
            g[y][x] = '?'
g = finish(g, (19, 11), [(8, 2, 'b'), (14, 6, 'b'), (5, 10, 'b'), (16, 4, 'g'), (10, 8, 'g'), (17, 10, 'g')])
write('02-beetle-rows.txt', 'Beetle Rows', [
    "Ladybeetles march up and down the tree rows.",
    "They never turn until something stops them.",
    "Put something in their way.",
], 'clear', 150, g)

# 1-3 Hedge Maze: a maze of walls; hedgehogs hide as hedges; duds to set off
g = [['#' for _ in range(21)] for _ in range(15)]
seen = {(1, 1)}
stack = [(1, 1)]
g[1][1] = '.'
while stack:
    x, y = stack[-1]
    nb = [(x + dx, y + dy, dx, dy) for dx, dy in [(2, 0), (-2, 0), (0, 2), (0, -2)] if 0 < x + dx < 20 and 0 < y + dy < 14 and (x + dx, y + dy) not in seen]
    if not nb:
        stack.pop()
        continue
    nx, ny, dx, dy = rnd.choice(nb)
    g[y + dy // 2][x + dx // 2] = '.'
    g[ny][nx] = '.'
    seen.add((nx, ny))
    stack.append((nx, ny))
for _ in range(30):
    x, y = rnd.randrange(1, 20), rnd.randrange(1, 14)
    if g[y][x] == '#' and (x % 2) != (y % 2):
        g[y][x] = '.'
for y in range(1, 14):
    for x in range(1, 20):
        if g[y][x] == '.' and rnd.random() < 0.4:
            g[y][x] = '?'
g = finish(g, (19, 13), [(9, 3, 'h'), (15, 7, 'h'), (5, 11, 'h'), (11, 9, 'h'), (13, 1, 'g'), (3, 7, 'g')],
           duds=[(7, 5), (17, 3), (13, 11), (3, 13), (17, 11)])
write('03-hedge-maze.txt', 'Hedge Maze', [
    "Some of these hedges have eyes.",
    "Five of your duds came down in the maze.",
    "Set every one of them off, and the hatch opens.",
], 'duds', 180, g)

# 1-4 Frog Pond: a pond (water as wall) in the middle; frogs leave eggs
g = base(23, 15)
for y in range(5, 10):
    for x in range(7, 16):
        if (x - 11) ** 2 / 20 + (y - 7) ** 2 / 5.5 <= 1:
            g[y][x] = 'w'
for y in range(3, 12):
    for x in range(5, 18):
        if g[y][x] == '?' and (x - 11) ** 2 / 36 + (y - 7) ** 2 / 14 <= 1:
            g[y][x] = '.'
g = finish(g, (21, 13), [(6, 4, 'f'), (16, 4, 'f'), (6, 10, 'f'), (16, 10, 'f'), (11, 3, 'f'), (19, 7, 'g'), (3, 11, 'g')])
write('04-frog-pond.txt', 'Frog Pond', [
    "Frogs from the pond hop clean over the hedges.",
    "Beat one and it leaves an egg behind. Walk over it,",
    "and ride what hatches: the ride button (Y, E or .) hops.",
    "A mount takes a hit for you, too.",
], 'clear', 180, g)

# 1-5 Cider Press: belts carrying apples (and anything else) round the press
g = base(23, 15)
for x in range(4, 19):
    g[4][x] = '>'
for y in range(4, 10):
    g[y][19] = 'v'
for x in range(4, 20):
    g[10][x] = '<'
for y in range(5, 11):
    g[y][3] = '^'
g[4][3] = '>'
g[10][19] = '<'
g[10][3] = '^'
line(g, 9, 6, 13, 8, '#')
line(g, 8, 6, 8, 8, '.')
line(g, 14, 6, 14, 8, '.')
g = finish(g, (21, 13), [(6, 2, 'b'), (16, 12, 'b'), (11, 2, 'f'), (6, 12, 'f'), (17, 7, 'g')],
           duds=[(1, 7), (21, 5), (11, 13), (5, 7), (17, 1), (21, 9)])
write('05-cider-press.txt', 'Cider Press', [
    "The cider press still turns, and its belts carry anything",
    "left on them. Six duds rode in with the apples.",
], 'duds', 180, g)

# 1-6 Ram Pasture: open pasture behind fences
g = base(25, 15, pillars=False, fill='.')
for x in range(3, 22):
    if x % 6 != 0:
        g[5][x] = '#'
        g[9][x] = '#'
for y in range(2, 13):
    for x in range(1, 24):
        if g[y][x] == '.' and rnd.random() < 0.22:
            g[y][x] = '?'
for (x, y) in [(6, 3), (18, 3), (12, 7), (6, 11), (18, 11)]:
    g[y][x] = '#'
g = finish(g, (23, 13), [(10, 3, 'r'), (20, 7, 'r'), (8, 11, 'r'), (16, 11, 'r'), (14, 3, 'g'), (4, 7, 'g')])
write('06-ram-pasture.txt', 'Ram Pasture', [
    "Rams graze the pasture. Step into a ram's line",
    "and it charges until something stops it.",
    "Tame one and you can dash.",
], 'clear', 180, g)

# 1-7 Crow Tree: one great tree in the middle; crows fly over hedges
g = base(25, 17)
for y in range(4, 13):
    for x in range(8, 17):
        if (x - 12) ** 2 / 16 + (y - 8) ** 2 / 16 <= 1:
            g[y][x] = 'T'
        elif (x - 12) ** 2 / 30 + (y - 8) ** 2 / 26 <= 1 and g[y][x] == '?':
            g[y][x] = '.'
g = finish(g, (23, 15), [(5, 5, 'c'), (19, 5, 'c'), (5, 11, 'c'), (19, 11, 'c'), (12, 2, 'h'), (12, 14, 'h'), (21, 3, 'b'), (3, 13, 'b')])
write('07-crow-tree.txt', 'Crow Tree', [
    "Crows nest in the great tree, and they fly",
    "right over the hedges. They don't need a path to reach you.",
], 'clear', 180, g)

# 1-8 The Old Grove: everything together, and the last duds
g = base(27, 17)
for (cx, cy) in [(7, 5), (19, 5), (7, 11), (19, 11)]:
    for y in range(cy - 2, cy + 3):
        for x in range(cx - 2, cx + 3):
            if abs(x - cx) + abs(y - cy) <= 2 and g[y][x] == '?':
                g[y][x] = '.'
    g[cy][cx] = '#'
g = finish(g, (25, 15), [(13, 3, 'r'), (13, 13, 'r'), (5, 8, 'c'), (21, 8, 'c'), (9, 7, 'f'), (17, 9, 'f'), (11, 15, 'h'), (23, 1, 'h'), (15, 5, 'b')],
           duds=[(1, 15), (25, 1), (13, 8), (7, 3), (19, 13), (3, 9), (23, 7), (15, 15)])
write('08-the-old-grove.txt', 'The Old Grove', [
    "The oldest grove in the Orchard,",
    "where the rest of your duds came down.",
    "Everything that lives here is awake.",
], 'duds', 210, g)

# 1-9 The Bramblehog: an open arena for the boss (2x2 tiles, so no pillar grid: four
# pillars only, lanes two tiles wide for it to roll down)
g = base(19, 13, pillars=False, fill='.')
for y in range(1, 12):
    for x in range(1, 18):
        if rnd.random() < 0.16 and not (6 <= x <= 12 and 3 <= y <= 9):
            g[y][x] = '?'
for (x, y) in [(5, 4), (13, 4), (5, 8), (13, 8)]:
    g[y][x] = '#'
g[5][9] = 'x'
g = finish(g, (17, 11), [])
g[11][17] = '.'
ending = [
    "The Bramblehog uncurls, blinks at the sky full of your",
    "fireworks, and wanders off to sleep under the oldest tree.",
    "The Orchard is quiet again. But past the hills, something",
    "is ticking: more duds came down in the Clockworks.",
    "To be continued.",
]
write('09-bramblehog.txt', 'The Bramblehog', [
    "Something old guards the heart of the Orchard.",
    "It curls into a ball of brambles and rolls at whatever",
    "moves, and the grubs come when it calls.",
], 'boss', 240, g, ''.join(f"ending: {l}\n" for l in ending))
