use crate::raster::put_pixel;

#[inline]
fn edge(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

pub fn tri_fill(
    buf: &mut [u32],
    w: usize,
    h: usize,
    t: [(i32, i32); 3],
    color: u32,
) {
    let (x0, y0) = t[0];
    let (x1, y1) = t[1];
    let (x2, y2) = t[2];

    let min_x = x0.min(x1).min(x2).max(0) as i32;
    let min_y = y0.min(y1).min(y2).max(0) as i32;
    let max_x = x0.max(x1).max(x2).min((w-1) as i32);
    let max_y = y0.max(y1).max(y2).min((h-1) as i32);

    let area = edge(x0, y0, x1, y1, x2, y2);
    if area == 0 { return; }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge(x1, y1, x2, y2, x, y);
            let w1 = edge(x2, y2, x0, y0, x, y);
            let w2 = edge(x0, y0, x1, y1, x, y);

            // Triángulos en sentido CCW: aceptamos si los tres son >= 0 (o <= 0 si CW)
            if (w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0) {
                put_pixel(buf, w, h, x, y, color);
            }
        }
    }
}
