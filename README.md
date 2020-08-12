# autotyler

Autotyler is a program that turns small "incomplete" tilemaps into complete godot 3x3 minimal autotile tilemaps, including support for configuring edge sizes.

Licensed under the Apache license, version 2.0.

In: ![in_basic](examples/basic.png)

Out: ![out_basic](examples/basic_output.png)

There are five other algorithms with varying levels of sophistication, like below:

### basic_border

![in_basic](examples/basicborder.png)

![out_basic](examples/basicborder_output.png)

### 3x3

![in_basic](examples/3x3.png)

![out_basic](examples/3x3_output.png)

or with edges=8,6,8,10:

![out_basic](examples/3x3_output_edges_8,6,8,10.png)

### 4x4

![in_basic](examples/4x4.png)

![out_basic](examples/4x4_output.png)

### minitiles

![in_basic](examples/minitiles.png)

![out_basic](examples/minitiles_output.png)

![in_basic](examples/minitiles2.png)

![out_basic](examples/minitiles2_output.png)

The minitiles algorithm has the same input as https://github.com/lunarfyre7/GodotAutotileAssembler

## Usage
```
tilegen <infile> <outfile> <options> [tile list]

options:
  mode=basic | basic_border | 3x3 | 4x4 | minitiles
    The algorithm used to generate the tilemap.
      basic: 2 tiles, see examples. (default)
      basic_border: 2 tiles, see examples.
      3x3: 9 tiles, see examples.
      4x4: 16 tiles, see examples.
      minitiles: 5 tiles, see https://github.com/lunarfyre7/GodotAutotileAssembler and examples.
  size=N
    The height and width of the tile in pixels. Currently only support square tiles.
  edges=LEFT,TOP,RIGHT,BOTTOM | LEFT,TOP | LEFT
    The amount of space taken up by edges. Omitted dimensions are generated by subtracting the opposite dimension from the tile size, or by copying the adjacent edge. Only minitiles is guaranteed to work with edges settings where opposite edges don't add up to the tilesize.
  offset=N
    The offset, in TILES (not pixels), from the top left corner of the screen from which to search for tiles. Useful for using the same input tilesheet to generate many tilemaps.
  border=N
    Used exclusively by the basic_border algorithm. The default is 2.

Do not place spaces around the = when specifying options.

tile list:
  The basic, basic_border, and minitiles modes allow you to list arbitrary tile coordinates to grab tiles from. Support for this will be added to the 3x3 and 4x4 modes later on.
  Example:
    tilegen basic.png out.png size=32 offset=4,1 0,0 0,2
This gets the first tile from 4,1 and the second tile from 4,3.

The tile list is optional.```
