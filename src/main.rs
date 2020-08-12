use std::env;

#[allow(unused_mut)]

use image::GenericImageView;
use image::Pixel;
use core::cmp;
use std::cell::RefCell;

fn main()
{
    let mut in_filename = String::new();
    let mut out_filename = String::new();
    let mut size = 16;
    let mut _gap = 0;
    let mut mode = "basic".to_string();
    let mut border = 2;
    // also: minitiles, extended, etc
    
    let mut left_edge = 8;
    let mut right_edge = 8;
    let mut top_edge = 8;
    let mut bottom_edge = 8;
    
    let mut specified_edges = false;
    let mut special_edges = false;
    
    let mut offset_x = 0;
    let mut offset_y = 0;
    
    let mut origin_tile_list = Vec::<(u32, u32)>::new();
    
    for arg in env::args().skip(1)
    {
        if in_filename.len() == 0
        {
            in_filename = arg.clone();
            continue;
        }
        if out_filename.len() == 0
        {
            out_filename = arg.clone();
            continue;
        }
        
        let parse = arg.splitn(2, "=").collect::<Vec<_>>();
        if parse.len() == 2
        {
            match parse[0]
            {
                "size" =>
                {
                    size = parse[1].parse::<u32>().unwrap();
                    if !specified_edges
                    {
                        left_edge = size/2;
                        right_edge = size-left_edge;
                        top_edge = size/2;
                        bottom_edge = size-top_edge;
                    }
                    if special_edges
                    {
                        println!("please specify size before edge measures");
                    }
                }
                "edges" =>
                {
                    let dims = parse[1].splitn(4, ",").collect::<Vec<_>>();
                    specified_edges = true;
                    if dims.len() >= 4
                    {
                        left_edge = dims[0].parse::<u32>().unwrap();
                        top_edge = dims[1].parse::<u32>().unwrap();
                        right_edge = dims[2].parse::<u32>().unwrap();
                        bottom_edge = dims[3].parse::<u32>().unwrap();
                        continue;
                    }
                    special_edges = true;
                    if dims.len() >= 2
                    {
                        left_edge = dims[0].parse::<u32>().unwrap();
                        top_edge = dims[1].parse::<u32>().unwrap_or(left_edge);
                        right_edge = size-left_edge;
                        bottom_edge = size-top_edge;
                    }
                    else
                    {
                        top_edge = dims[0].parse::<u32>().unwrap();
                        bottom_edge = size-top_edge;
                    }
                }
                "offset" =>
                {
                    let dims = parse[1].splitn(2, ",").collect::<Vec<_>>();
                    offset_x = dims[0].parse::<u32>().unwrap();
                    offset_y = dims[1].parse::<u32>().unwrap();
                }
                "gap" => panic!("gap not implemented yet"),
                "mode" => mode = parse[1].to_string(),
                "border" => border = parse[1].parse::<u32>().unwrap(),
                _ => panic!("unsupported option {}", parse[0]),
            }
            continue;
        }
        let parse = arg.splitn(2, ",").collect::<Vec<_>>();
        if parse.len() == 2
        {
            origin_tile_list.push((parse[0].parse::<u32>().unwrap(), parse[1].parse::<u32>().unwrap()));
        }
    }
    if out_filename == ""
    {
        println!(
"usage:
autotyler <infile> <outfile> <options> [tile list]

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
    autotyler basic.png out.png size=32 offset=4,1 0,0 0,2
This gets the first tile from 4,1 and the second tile from 4,3.

The tile list is optional.
");
        return;
    }
    
    let in_img = RefCell::new(image::open(in_filename).unwrap());
    
    if origin_tile_list.is_empty()
    {
        if mode != "minitiles"
        {
            origin_tile_list.push((0, 0));
            if in_img.borrow().dimensions().1 >= size*2
            {
                origin_tile_list.push((0, 1));
            }
            else
            {
                origin_tile_list.push((1, 0));
            }
        }
        else
        {
            origin_tile_list.push((0, 0));
            origin_tile_list.push((1, 0));
            origin_tile_list.push((2, 0));
            origin_tile_list.push((3, 0));
            origin_tile_list.push((4, 0));
        }
    }
    for r in origin_tile_list.iter_mut()
    {
        r.0 += offset_x;
        r.1 += offset_y;
    }
    
    let out_img = RefCell::new(image::RgbaImage::new(12*size, 4*size));
    
    let copy_tile_ext = |(mut x_in, mut y_in) : (u32, u32), (mut x_out, mut y_out) : (u32, u32), (min_x, min_y, mut max_x, mut max_y) : (u32, u32, u32, u32), column : Option<u32>, row : Option<u32>|
    {
        x_in *= size;
        y_in *= size;
        x_out *= size;
        y_out *= size;
        max_x = size-max_x;
        max_y = size-max_y;
        for ix in cmp::max(min_x, 0)..cmp::min(max_x, size)
        {
            for iy in cmp::max(min_y, 0)..cmp::min(max_y, size)
            {
                out_img.borrow_mut().put_pixel(x_out+ix, y_out+iy, in_img.borrow().get_pixel(x_in+column.unwrap_or(ix), y_in+row.unwrap_or(iy)).to_rgba());
            }
        }
    };
    let copy_tile_part = |a, b, c| copy_tile_ext(a, b, c, None, None);
    let copy_tile = |a, b| copy_tile_part(a, b, (0, 0, 0, 0));
    //let copy_column = |a, b, column| copy_tile_ext(a, b, (0, 0, 0, 0), Some(column), None);
    //let copy_row = |a, b, row| copy_tile_ext(a, b, (0, 0, 0, 0), None, Some(row));
    
    let copy_tile_inplace = |(mut x_in, mut y_in) : (u32, u32), (mut x_out, mut y_out) : (u32, u32)|
    {
        x_in *= size;
        y_in *= size;
        x_out *= size;
        y_out *= size;
        for ix in 0..size
        {
            for iy in 0..size
            {
                let px = out_img.borrow_mut().get_pixel(x_in+ix, y_in+iy).to_rgba();
                out_img.borrow_mut().put_pixel(x_out+ix, y_out+iy, px);
            }
        }
    };
    let clear_tile = |(mut x_out, mut y_out) : (u32, u32)|
    {
        x_out *= size;
        y_out *= size;
        for ix in 0..size
        {
            for iy in 0..size
            {
                let px = image::Rgba::from([0, 0, 0, 0]);
                out_img.borrow_mut().put_pixel(x_out+ix, y_out+iy, px);
            }
        }
    };
    
    let copy_4x4_to_12x4 = ||
    {
        copy_tile_inplace((2, 1), (5, 1));
        copy_tile_inplace((2, 1), (5, 2));
        copy_tile_inplace((2, 1), (6, 1));
        copy_tile_inplace((2, 1), (6, 2));
        
        copy_tile_inplace((2, 1), (9, 1));
        copy_tile_inplace((2, 1), (9, 2));
        // blank
        copy_tile_inplace((2, 1), (10, 2));
        
        copy_tile_inplace((2, 1), (9, 0));
        copy_tile_inplace((2, 1), (11, 1));
        copy_tile_inplace((2, 1), (8, 2));
        copy_tile_inplace((2, 1), (10, 3));
        
        copy_tile_inplace((2, 1), (4, 0));
        copy_tile_inplace((2, 1), (7, 0));
        copy_tile_inplace((2, 1), (4, 3));
        copy_tile_inplace((2, 1), (7, 3));
        
        
        copy_tile_inplace((1, 0), (8, 0));
        copy_tile_inplace((1, 2), (8, 3));
        copy_tile_inplace((3, 0), (11, 0));
        copy_tile_inplace((3, 2), (11, 3));
        
        copy_tile_inplace((2, 0), (5, 0));
        copy_tile_inplace((2, 0), (6, 0));
        copy_tile_inplace((2, 0), (10, 0));
        
        copy_tile_inplace((2, 2), (5, 3));
        copy_tile_inplace((2, 2), (6, 3));
        copy_tile_inplace((2, 2), (9, 3));
        
        copy_tile_inplace((1, 1), (4, 1));
        copy_tile_inplace((1, 1), (4, 2));
        copy_tile_inplace((1, 1), (8, 1));
        
        copy_tile_inplace((3, 1), (7, 1));
        copy_tile_inplace((3, 1), (7, 2));
        copy_tile_inplace((3, 1), (11, 2));
        
        clear_tile((10, 1));
    };
    
    let m_left = size-right_edge;
    let m_right = size-left_edge;
    let m_top = size-bottom_edge;
    let m_bottom = size-top_edge;
    
    match mode.as_str()
    {
        "basic" =>
        {
            let tile_a = origin_tile_list[0];
            let tile_b = origin_tile_list[1];
            
            for ix in 0..12
            {
                for iy in 0..4
                {
                    copy_tile(tile_a, (ix, iy));
                }
            }
            
            copy_tile(tile_b, (0, 3));
            
            copy_tile_part(tile_b, (0, 0), (0, 0, 0, m_bottom));
            copy_tile_part(tile_b, (0, 2), (0, m_top, 0, 0));
            
            copy_tile_part(tile_b, (1, 3), (0, 0, m_right, 0));
            copy_tile_part(tile_b, (3, 3), (m_left, 0, 0, 0));
            
            copy_tile_part(tile_b, (1, 0), (0, 0, m_right, m_bottom));
            copy_tile_part(tile_b, (1, 2), (0, m_top, m_right, 0));
            copy_tile_part(tile_b, (3, 0), (m_left, 0, 0, m_bottom));
            copy_tile_part(tile_b, (3, 2), (m_left, m_top, 0, 0));
            
            copy_4x4_to_12x4();
        }
        "basic_border" =>
        {
            let tile_a = origin_tile_list[0];
            let tile_b = origin_tile_list[1];
            
            let m_border = size-border;
            let m_left_border = size-m_right;
            let m_top_border = size-m_bottom;
            let m_right_border = size-m_left;
            let m_bottom_border = size-m_top;
            
            for ix in 0..12
            {
                for iy in 0..4
                {
                    copy_tile(tile_a, (ix, iy));
                }
            }
            
            copy_tile(tile_b, (0, 3));
            
            copy_tile_part(tile_b, (0, 0), (0, 0, 0, m_bottom));
            copy_tile_part(tile_b, (0, 2), (0, m_top, 0, 0));
            
            copy_tile_part(tile_b, (1, 3), (0, 0, m_right, 0));
            copy_tile_part(tile_b, (3, 3), (m_left, 0, 0, 0));
            
            copy_tile_part(tile_b, (1, 0), (0, 0, m_right, m_bottom));
            copy_tile_part(tile_b, (1, 2), (0, m_top, m_right, 0));
            copy_tile_part(tile_b, (3, 0), (m_left, 0, 0, m_bottom));
            copy_tile_part(tile_b, (3, 2), (m_left, m_top, 0, 0));
            
            
            copy_tile_ext(tile_b, (0, 0), (0, m_top_border, m_border, 0), None, Some(m_top));
            copy_tile_ext(tile_b, (0, 0), (m_border, m_top_border, 0, 0), None, Some(m_top));
            
            copy_tile_ext(tile_b, (0, 1), (0, 0, m_border, 0), None, Some(m_top));
            copy_tile_ext(tile_b, (0, 1), (m_border, 0, 0, 0), None, Some(m_top));
            
            copy_tile_ext(tile_b, (0, 2), (0, 0, m_border, m_bottom_border), None, Some(m_top));
            copy_tile_ext(tile_b, (0, 2), (m_border, 0, 0, m_bottom_border), None, Some(m_top));
            
            
            copy_tile_ext(tile_b, (1, 3), (m_left_border, 0, 0, m_border), Some(m_left), None);
            copy_tile_ext(tile_b, (1, 3), (m_left_border, m_border, 0, 0), Some(m_left), None);
            
            copy_tile_ext(tile_b, (2, 3), (0, 0, 0, m_border), Some(m_left), None);
            copy_tile_ext(tile_b, (2, 3), (0, m_border, 0, 0), Some(m_left), None);
            
            copy_tile_ext(tile_b, (3, 3), (0, 0, m_right_border, m_border), Some(m_left), None);
            copy_tile_ext(tile_b, (3, 3), (0, m_border, m_right_border, 0), Some(m_left), None);
            
            
            copy_tile_ext(tile_b, (1, 0), (m_left_border, 0, 0, m_border), Some(m_left), None);
            copy_tile_ext(tile_b, (1, 0), (0, m_top_border, m_border, 0), None, Some(m_top));
            
            copy_tile_ext(tile_b, (1, 1), (0, 0, m_border, 0), None, Some(m_top));
            
            copy_tile_ext(tile_b, (1, 2), (0, 0, m_border, m_bottom_border), None, Some(m_top));
            copy_tile_ext(tile_b, (1, 2), (m_left_border, m_border, 0, 0), Some(m_left), None);
            
            
            copy_tile_ext(tile_b, (2, 0), (0, 0, 0, m_border), Some(m_left), None);
            
            copy_tile_ext(tile_b, (2, 2), (0, m_border, 0, 0), Some(m_left), None);
            
            
            copy_tile_ext(tile_b, (3, 0), (0, 0, m_right_border, m_border), Some(m_left), None);
            copy_tile_ext(tile_b, (3, 0), (m_border, m_top_border, 0, 0), None, Some(m_top));
            
            copy_tile_ext(tile_b, (3, 1), (m_border, 0, 0, 0), None, Some(m_top));
            
            copy_tile_ext(tile_b, (3, 2), (m_border, 0, 0, m_bottom_border), None, Some(m_top));
            copy_tile_ext(tile_b, (3, 2), (0, m_border, m_right_border, 0), Some(m_left), None);
            
            copy_4x4_to_12x4();
        }
        "3x3" =>
        {
            let xm_left = left_edge;
            let xm_top = top_edge;
            let xm_right = right_edge;
            let xm_bottom = bottom_edge;
            
            for ix in 0..size*3
            {
                for iy in 0..size*3
                {
                    out_img.borrow_mut().put_pixel(ix+size, iy, in_img.borrow().get_pixel(ix, iy).to_rgba());
                }
            }
            copy_tile((1, 1), (0, 1));
            copy_tile((1, 1), (0, 3));
            copy_tile((1, 1), (2, 3));
            
            copy_tile_part((1, 0), (0, 3), (0, 0, 0, m_bottom));
            copy_tile_part((1, 2), (0, 3), (0, m_top, 0, 0));
            copy_tile_part((0, 1), (0, 3), (0, 0, m_right, 0));
            copy_tile_part((2, 1), (0, 3), (m_left, 0, 0, 0));
            
            copy_tile_part((2, 2), (0, 3), (m_left, m_top, 0, 0));
            copy_tile_part((0, 2), (0, 3), (0, m_top, m_right, 0));
            copy_tile_part((0, 0), (0, 3), (0, 0, m_right, m_bottom));
            copy_tile_part((2, 0), (0, 3), (m_left, 0, 0, m_bottom));
            
            copy_tile_inplace((0, 3), (0, 0));
            copy_tile_inplace((0, 3), (0, 2));
            copy_tile_inplace((0, 3), (1, 3));
            copy_tile_inplace((0, 3), (3, 3));
            copy_tile_part((1, 1), (0, 0), (0, xm_top, 0, 0));
            copy_tile_part((1, 1), (0, 2), (0, 0, 0, xm_bottom));
            copy_tile_part((1, 1), (1, 3), (xm_left, 0, 0, 0));
            copy_tile_part((1, 1), (3, 3), (0, 0, xm_right, 0));
            
            
            copy_tile_part((2, 1), (0, 0), (m_left, xm_top, 0, 0));
            copy_tile_part((0, 1), (0, 0), (0, xm_top, m_right, 0));
            
            copy_tile_part((2, 1), (0, 1), (m_left, 0, 0, 0));
            copy_tile_part((0, 1), (0, 1), (0, 0, m_right, 0));
            
            copy_tile_part((2, 1), (0, 2), (m_left, 0, 0, xm_bottom));
            copy_tile_part((0, 1), (0, 2), (0, 0, m_right, xm_bottom));
            
            
            copy_tile_part((1, 2), (1, 3), (xm_left, m_top, 0, 0));
            copy_tile_part((1, 0), (1, 3), (xm_left, 0, 0, m_bottom));
            
            copy_tile_part((1, 2), (2, 3), (0, m_top, 0, 0));
            copy_tile_part((1, 0), (2, 3), (0, 0, 0, m_bottom));
            
            copy_tile_part((1, 2), (3, 3), (0, m_top, xm_right, 0));
            copy_tile_part((1, 0), (3, 3), (0, 0, xm_right, m_bottom));
            
            copy_tile_part((2, 2), (3, 3), (0, m_top, 0, 0));
            copy_tile_part((0, 2), (1, 3), (0, m_top, 0, 0));
            
            copy_tile_part((2, 0), (3, 3), (0, 0, 0, m_bottom));
            copy_tile_part((0, 0), (1, 3), (0, 0, 0, m_bottom));
            
            copy_4x4_to_12x4();
        }
        "4x4" =>
        {
            for ix in 0..size*4
            {
                for iy in 0..size*4
                {
                    out_img.borrow_mut().put_pixel(ix, iy, in_img.borrow().get_pixel(ix, iy).to_rgba());
                }
            }
            copy_4x4_to_12x4();
        }
        "minitiles" =>
        {
            let xm_left = left_edge;
            let xm_top = top_edge;
            let xm_right = right_edge;
            let xm_bottom = bottom_edge;
            
            let tile_a = origin_tile_list[0];
            let tile_b = origin_tile_list[1];
            let tile_c = origin_tile_list[2];
            let tile_x = origin_tile_list[3];
            let tile_z = origin_tile_list[4];
            
            copy_tile(tile_a, (0, 3));
            copy_tile(tile_b, (0, 1));
            copy_tile(tile_c, (2, 3));
            copy_tile(tile_x, (2, 1));
            copy_tile(tile_z, (9, 2));
            
            copy_tile(tile_b, (0, 0));
            copy_tile_part(tile_a, (0, 0), (0, 0, 0, m_bottom));
            copy_tile(tile_b, (0, 2));
            copy_tile_part(tile_a, (0, 2), (0, m_top, 0, 0));
            
            copy_tile(tile_c, (1, 3));
            copy_tile_part(tile_a, (1, 3), (0, 0, m_right, 0));
            copy_tile(tile_c, (3, 3));
            copy_tile_part(tile_a, (3, 3), (m_left, 0, 0, 0));
            
            
            copy_tile(tile_a, (1, 0));
            copy_tile_part(tile_b, (1, 0), (0, m_top, 0, 0));
            copy_tile_part(tile_c, (1, 0), (m_left, 0, 0, 0));
            copy_tile_part(tile_x, (1, 0), (m_left, m_top, 0, 0));
            
            copy_tile(tile_c, (2, 0));
            copy_tile_part(tile_x, (2, 0), (0, m_top, 0, 0));
            
            copy_tile(tile_a, (3, 0));
            copy_tile_part(tile_b, (3, 0), (0, m_top, 0, 0));
            copy_tile_part(tile_c, (3, 0), (0, 0, m_right, 0));
            copy_tile_part(tile_x, (3, 0), (0, m_top, m_right, 0));
            
            
            copy_tile(tile_b, (1, 1));
            copy_tile_part(tile_x, (1, 1), (m_left, 0, 0, 0));
            
            copy_tile(tile_b, (3, 1));
            copy_tile_part(tile_x, (3, 1), (0, 0, m_right, 0));
            
            
            
            copy_tile(tile_a, (1, 2));
            copy_tile_part(tile_b, (1, 2), (0, 0, 0, m_bottom));
            copy_tile_part(tile_c, (1, 2), (m_left, 0, 0, 0));
            copy_tile_part(tile_x, (1, 2), (m_left, 0, 0, m_bottom));
            
            copy_tile(tile_c, (2, 2));
            copy_tile_part(tile_x, (2, 2), (0, 0, 0, m_bottom));
            
            copy_tile(tile_a, (3, 2));
            copy_tile_part(tile_b, (3, 2), (0, 0, 0, m_bottom));
            copy_tile_part(tile_c, (3, 2), (0, 0, m_right, 0));
            copy_tile_part(tile_x, (3, 2), (0, 0, m_right, m_bottom));
            
            
            copy_4x4_to_12x4();
            
            
            copy_tile(tile_x, (4, 0));
            copy_tile(tile_x, (7, 0));
            copy_tile(tile_x, (4, 3));
            copy_tile(tile_x, (7, 3));
            
            copy_tile(tile_z, (5, 1));
            copy_tile(tile_z, (6, 1));
            copy_tile(tile_z, (5, 2));
            copy_tile(tile_z, (6, 2));
            
            copy_tile_part(tile_z, (4, 0), (0, 0, xm_right, xm_bottom));
            copy_tile_part(tile_z, (7, 0), (xm_left, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (4, 3), (0, xm_top, xm_right, 0));
            copy_tile_part(tile_z, (7, 3), (xm_left, xm_top, 0, 0));
            
            copy_tile_part(tile_x, (5, 1), (0, 0, m_right, m_bottom));
            copy_tile_part(tile_x, (6, 1), (m_left, 0, 0, m_bottom));
            copy_tile_part(tile_x, (5, 2), (0, m_top, m_right, 0));
            copy_tile_part(tile_x, (6, 2), (m_left, m_top, 0, 0));
            
            copy_tile_part(tile_z, (5, 0), (xm_left, xm_top, 0, 0));
            copy_tile_part(tile_z, (6, 0), (0, xm_top, xm_right, 0));
            copy_tile_part(tile_z, (5, 3), (xm_left, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (6, 3), (0, 0, xm_right, xm_bottom));
            copy_tile_part(tile_z, (4, 1), (xm_left, xm_top, 0, 0));
            copy_tile_part(tile_z, (7, 1), (0, xm_top, xm_right, 0));
            copy_tile_part(tile_z, (4, 2), (xm_left, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (7, 2), (0, 0, xm_right, xm_bottom));
            
            
            
            copy_tile_part(tile_z, (8, 0), (xm_left, xm_top, 0, 0));
            copy_tile_part(tile_z, (9, 0), (0, xm_top, 0, 0));
            copy_tile_part(tile_z, (10, 0), (0, xm_top, 0, 0));
            copy_tile_part(tile_z, (11, 0), (0, xm_top, xm_right, 0));
            
            copy_tile_part(tile_z, (8, 3), (xm_left, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (9, 3), (0, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (10, 3), (0, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (11, 3), (0, 0, xm_right, xm_bottom));
            
            
            copy_tile_part(tile_z, (8, 1), (xm_left, 0, 0, 0));
            copy_tile_part(tile_z, (8, 2), (xm_left, 0, 0, 0));
            
            copy_tile_part(tile_z, (11, 1), (0, 0, xm_right, 0));
            copy_tile_part(tile_z, (11, 2), (0, 0, xm_right, 0));
            
            copy_tile_part(tile_z, (9, 1), (xm_left, 0, 0, xm_bottom));
            copy_tile_part(tile_z, (9, 1), (0, xm_top, xm_right, 0));
            
            copy_tile_part(tile_z, (10, 2), (0, 0, xm_right, xm_bottom));
            copy_tile_part(tile_z, (10, 2), (xm_left, xm_top, 0, 0));
            
            
            copy_tile(tile_z, (9, 2));
            clear_tile((10, 1));
        }
        _ => panic!("unknown mode {}", mode)
    }
    
    out_img.into_inner().save(out_filename).unwrap();
}
