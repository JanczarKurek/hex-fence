# Hex Grid Notes

This project uses **axial coordinates** `(q, r)` with implicit third cube axis:

- `s = -q - r`

A tile is on board when all 3 axes are within radius:

- `|q| <= R`
- `|r| <= R`
- `|s| <= R`

## Direction Indexes

`neighbor_in_direction(d)` uses `d % 6` with this order:

0. `(q + 1, r)`
1. `(q + 1, r - 1)`
2. `(q, r - 1)`
3. `(q - 1, r)`
4. `(q - 1, r + 1)`
5. `(q, r + 1)`

`orientation` values in fence placement are these same indexes.

## Fence Shape Construction

Given anchor `A`, orientation `o`, and:

- `n0 = A.neighbor_in_direction(o)`
- `n1 = A.neighbor_in_direction(o + 1)`

Shapes are built as edge triples:

- `C`: `A-n0`, `A-n1`, `A-neighbor(o+2)`
- `Y`: `A-n0`, `A-n1`, `n0-n1`
- `S`: `A-n0`, `A-n1`, `n1-neighbor(o+3)`
- `S-mirror`: `A-n0`, `A-n1`, `n0-neighbor(o+4)`

Notes:

- `EdgeKey::from_cells` stores edges in sorted `(q, r)` order; edge direction is not significant.
- Fence validity checks enforce each edge is between adjacent cells and all players still have a path.
