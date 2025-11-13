use crate::raster::put_pixel;

#[inline]
fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

pub fn tri_fill_z(
    color: u32,
    buf: &mut [u32],
    depth: &mut [f32],
    w: usize,
    h: usize,
    v: [(f32, f32, f32); 3], // (x,y,z) en pantalla, z normalizada [0,1]
) {
    let (x0,y0,z0) = v[0];
    let (x1,y1,z1) = v[1];
    let (x2,y2,z2) = v[2];

    let min_x = (x0.min(x1).min(x2).floor().max(0.0)) as i32;
    let min_y = (y0.min(y1).min(y2).floor().max(0.0)) as i32;
    let max_x = (x0.max(x1).max(x2).ceil().min((w - 1) as f32)) as i32;
    let max_y = (y0.max(y1).max(y2).ceil().min((h - 1) as f32)) as i32;

    let area = edge(x0,y0,x1,y1,x2,y2);
    if area == 0.0 { return; }

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;

            let w0 = edge(x1,y1,x2,y2,px,py);
            let w1 = edge(x2,y2,x0,y0,px,py);
            let w2 = edge(x0,y0,x1,y1,px,py);

            if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                let inv = 1.0 / area;
                let b0 = w0 * inv;
                let b1 = w1 * inv;
                let b2 = w2 * inv;

                let z = b0 * z0 + b1 * z1 + b2 * z2;

                let idx = y as usize * w + x as usize;
                if z < depth[idx] {
                    depth[idx] = z;
                    put_pixel(buf, w, h, x, y, color);
                }
            }
        }
    }
}
