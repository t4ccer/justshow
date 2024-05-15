use crate::{Canvas, Color, Vector2, BYTES_PER_PIXEL};
use just_bdf::Glyph;

#[inline(always)]
pub fn set_pixel(buf: &mut [u8], window_size: Vector2<u32>, x: u32, y: u32, color: Color) {
    let offset = (window_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;

    #[cfg(debug_assertions)]
    {
        if x >= window_size.x {
            panic!("Point out of range: x = {x} >= {}, y = {y}", window_size.x)
        }
        if y >= window_size.y {
            panic!("Point out of range: x = {x}, y = {y} >= {}", window_size.y)
        }
    }

    buf[offset + 0] = color.b;
    buf[offset + 1] = color.g;
    buf[offset + 2] = color.r;
    buf[offset + 3] = color.a;
}

#[inline(always)]
pub fn blend_pixel(buf: &mut [u8], window_size: Vector2<u32>, x: u32, y: u32, color: Color) {
    let offset = (window_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;

    #[cfg(debug_assertions)]
    {
        if x >= window_size.x {
            panic!("Point out of range: x = {x} >= {}, y = {y}", window_size.x)
        }
        if y >= window_size.y {
            panic!("Point out of range: x = {x}, y = {y} >= {}", window_size.y)
        }
    }

    let old = Color {
        b: buf[offset + 0],
        g: buf[offset + 1],
        r: buf[offset + 2],
        a: buf[offset + 3],
    };

    let blended = Color::blend(old, color);

    buf[offset + 0] = blended.b;
    buf[offset + 1] = blended.g;
    buf[offset + 2] = blended.r;
    buf[offset + 3] = blended.a;
}

macro_rules! define_rectangle {
    ($canvas:expr, $position:expr, $size:expr, $color:expr, $set_pixel:ident) => {
        let window_size = $canvas.window_size();
        let buf = $canvas.raw_buf_mut();

        for cy in $position.y..($position.y + $size.y).clamp(0, window_size.y) {
            for cx in $position.x..($position.x + $size.x).clamp(0, window_size.x) {
                $set_pixel(buf, window_size, cx, cy, $color);
            }
        }
    };
}

#[inline]
pub fn rectangle_replace(
    canvas: &mut Canvas,
    position: Vector2<u32>,
    size: Vector2<u32>,
    color: Color,
) {
    define_rectangle!(canvas, position, size, color, set_pixel);
}

#[inline]
pub fn rectangle_blend(
    canvas: &mut Canvas,
    position: Vector2<u32>,
    size: Vector2<u32>,
    color: Color,
) {
    define_rectangle!(canvas, position, size, color, blend_pixel);
}

#[inline]
pub fn circle_replace(ui: &mut Canvas, center: Vector2<u32>, r: u32, color: Color) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    let x = center.x.saturating_sub(r);
    let y = center.y.saturating_sub(r);

    for cy in y..(y + r * 2).clamp(0, window_size.y) {
        for cx in x..(x + r * 2).clamp(0, window_size.x) {
            let point = Vector2 { x: cx, y: cy };
            if inside_circle(center, r, point) {
                set_pixel(buf, window_size, cx, cy, color);
            }
        }
    }
}

const CIRCLE_AA_RES: u32 = 3;
const CIRCLE_AA_PAD: f32 = 1.0 / (CIRCLE_AA_RES as f32 + 1.0);

#[inline]
pub fn circle_blend_with_anti_aliasing(
    ui: &mut Canvas,
    center: Vector2<u32>,
    radius: u32,
    color: Color,
) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    let x = center.x.saturating_sub(radius);
    let y = center.y.saturating_sub(radius);

    let r2 = radius as f32 * radius as f32;

    for current_y in y..(y + radius * 2).clamp(0, window_size.y) {
        for current_x in x..(x + radius * 2).clamp(0, window_size.x) {
            let mut count = 0;
            for subpixel_offset_x in 0..CIRCLE_AA_RES {
                for subpixel_offset_y in 0..CIRCLE_AA_RES {
                    let subpixel_x: f32 =
                        current_x as f32 + CIRCLE_AA_PAD * (1.0 + subpixel_offset_x as f32);
                    let subpixel_y: f32 =
                        current_y as f32 + CIRCLE_AA_PAD * (1.0 + subpixel_offset_y as f32);

                    let dx: f32 = subpixel_x - center.x as f32;
                    let dy: f32 = subpixel_y - center.y as f32;
                    count += (dx * dx + dy * dy <= r2) as u32;
                }
            }

            let mut c = color;
            let mut a = c.a as u32;
            a *= count;
            a /= CIRCLE_AA_RES * CIRCLE_AA_RES;
            c.a = a as u8;
            blend_pixel(buf, window_size, current_x, current_y, c);
        }
    }
}

#[inline]
pub fn thin_line(ui: &mut Canvas, start: Vector2<u32>, end: Vector2<u32>, color: Color) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    for (x, y) in LineIter::new(start, end) {
        if y >= window_size.y || x >= window_size.x {
            continue;
        }

        set_pixel(buf, window_size, x, y, color);
    }
}

#[inline]
pub fn thin_dashed_line(ui: &mut Canvas, start: Vector2<u32>, end: Vector2<u32>, color: Color) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    // chosen arbitrarily
    let dash_length: u32 = 10;
    let gap_length: u32 = 10;

    let mut n = 0;
    for (x, y) in LineIter::new(start, end) {
        if y >= window_size.y || x >= window_size.x {
            continue;
        }

        if n < dash_length {
            set_pixel(buf, window_size, x, y, color);
        }
        n += 1;
        if n >= dash_length + gap_length {
            n = 0;
        }
    }
}

#[derive(Debug)]
struct LineIter {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    dx: i32,
    dy: i32,
    curr_x: u32,
    curr_y: u32,
}

impl LineIter {
    fn new(start: Vector2<u32>, end: Vector2<u32>) -> Self {
        let Vector2 {
            x: mut x1,
            y: mut y1,
        } = start;

        let Vector2 {
            x: mut x2,
            y: mut y2,
        } = end;

        let dx = x2 as i32 - x1 as i32;
        let dy = y2 as i32 - y1 as i32;

        if dx.abs() > dy.abs() {
            if x1 > x2 {
                core::mem::swap(&mut x1, &mut x2);
                core::mem::swap(&mut y1, &mut y2);
            }
        } else if y1 > y2 {
            core::mem::swap(&mut x1, &mut x2);
            core::mem::swap(&mut y1, &mut y2);
        }

        Self {
            x1,
            y1,
            x2,
            y2,
            dx,
            dy,
            curr_x: x1,
            curr_y: y1,
        }
    }
}

impl Iterator for LineIter {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.dx == 0 && self.dy == 0 {
            return None;
        }

        if self.dx.abs() > self.dy.abs() {
            if self.curr_x > self.x2 {
                return None;
            }

            let y =
                (self.dy * (self.curr_x as i32 - self.x1 as i32) / self.dx + self.y1 as i32) as u32;
            let res = (self.curr_x, y);

            self.curr_x += 1;

            Some(res)
        } else {
            if self.curr_y > self.y2 {
                return None;
            }

            let x =
                (self.dx * (self.curr_y as i32 - self.y1 as i32) / self.dy + self.x1 as i32) as u32;
            let res = (x, self.curr_y);

            self.curr_y += 1;

            Some(res)
        }
    }
}

// TODO: text_bdf_bounding_box

#[inline]
pub fn text_bdf_width<'a>(font: impl Fn(char) -> &'a Glyph, size: u32, text: &str) -> u32 {
    let mut x = 0;
    for glyph in text.chars().map(font) {
        x += size * glyph.bounding_box.width + size * 2;
    }
    x
}

pub fn glyph_bdf(ui: &mut Canvas, position: Vector2<u32>, size: u32, glyph: &Glyph, color: Color) {
    let padded_width = ((glyph.bounding_box.width + 7) / 8) * 8;
    let padded_height = ((glyph.bounding_box.height + 7) / 8) * 8;

    let x_off = padded_width as i32;
    let y_off = (padded_height - glyph.bounding_box.height) as i32 - 1;

    let total_x_offset = position.x as i32 + (x_off - glyph.bounding_box.x_off) * size as i32;
    let total_y_offset = position.y as i32 + (y_off - glyph.bounding_box.y_off) * size as i32;

    for gy in 0u32..glyph.bounding_box.height {
        for gx in 0u32..padded_width {
            let n = gy * padded_width + gx;
            let has_pixel = (glyph.bitmap[(n / 8) as usize] & (1 << (n % 8))) != 0;

            if has_pixel {
                rectangle_replace(
                    ui,
                    Vector2 {
                        x: (total_x_offset - (gx as i32 * size as i32)) as u32,
                        y: (total_y_offset + (gy as i32 * size as i32)) as u32,
                    },
                    Vector2 { x: size, y: size },
                    color,
                );
            }
        }
    }
}

#[inline]
pub fn distance_squared(p1: Vector2<u32>, p2: Vector2<u32>) -> u32 {
    let x_dist = (p1.x as i32 - p2.x as i32).unsigned_abs();
    let y_dist = (p1.y as i32 - p2.y as i32).unsigned_abs();
    x_dist * x_dist + y_dist * y_dist
}

#[inline]
pub fn inside_circle(center: Vector2<u32>, r: u32, point: Vector2<u32>) -> bool {
    distance_squared(center, point) <= r * r
}

#[inline]
pub fn inside_rectangle(position: Vector2<u32>, size: Vector2<u32>, point: Vector2<u32>) -> bool {
    point.x >= position.x
        && point.x <= position.x + size.x
        && point.y >= position.y
        && point.y <= size.y + position.y
}
