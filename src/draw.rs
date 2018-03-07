use super::*;
use super::util::*;

pub fn basic(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut sub: Rect) {
    /* Clip to destination buffer */
    clip_rect_offset(&mut sub, &mut x, &mut y, b.clip);
    /* Clipped off screen? */
    if sub.w <= 0 || sub.h <= 0 {
        return;
    }
    /* Draw */
    for iy in 0..(sub.h as usize) {
        let (mut d_off, mut s_off) = (0, 0);
        for _ in (0..sub.w).rev() {
            blend_pixel(
                &b.mode,
                &mut b.pixels[(x + (y + iy as i32) * b.w + d_off) as usize],
                src.pixels[(sub.x + (sub.y + iy as i32) * src.w + s_off) as usize],
            );
            d_off += 1;
            s_off += 1;
        }
    }
}

pub fn scaled(b: &mut Buffer, src: &Buffer, x: i32, y: i32, mut sub: Rect, t: Transform) {
    let abs_sx =
        if t.sx < 0.0 { -t.sx } else { t.sx };
    let abs_sy =
        if t.sy < 0.0 { -t.sy } else { t.sy };
    let mut width = (sub.w as f32 * abs_sx + 0.5).floor() as i32;
    let mut height = (sub.h as f32 * abs_sy + 0.5).floor() as i32;
    let osx = if t.sx < 0.0 {
        (sub.w << FX_BITS_12) - 1
    } else {
        0
    };
    let osy = if t.sy < 0.0 {
        (sub.h << FX_BITS_12) - 1
    } else {
        0
    };
    let ix = ((sub.w << FX_BITS_12) as f32 / t.sx / sub.w as f32) as i32;
    let iy = ((sub.h << FX_BITS_12) as f32 / t.sy / sub.h as f32) as i32;
    /* Adjust x/y depending on origin */
    let x = (x as f32
        - ((if t.sx < 0.0 { width } else { 0 }) - (if t.sx < 0.0 { -1 } else { 1 })) as f32 * t.ox
            * abs_sx) as i32;
    let y = (y as f32
        - ((if t.sy < 0.0 { height } else { 0 }) - (if t.sy < 0.0 { -1 } else { 1 })) as f32 * t.oy
            * abs_sy) as i32;
    /* Clipped completely offscreen horizontally? */
    if x + width < b.clip.x || x > b.clip.x + b.clip.w {
        return;
    }
    /* Adjust for clipping */
    let mut dy = 0;
    let mut odx = 0;
    let _d = b.clip.y - y;
    if _d > 0 {
        dy = _d;
        sub.y += (_d as f32 / t.sy) as i32;
    }
    let _d = b.clip.x - x;
    if _d > 0 {
        odx = _d;
        sub.x += (_d as f32 / t.sx) as i32;
    }
    let _d = (y + height) - (b.clip.y + b.clip.h);
    if _d > 0 {
        height -= _d;
    }
    let _d = (x + width) - (b.clip.x + b.clip.w);
    if _d > 0 {
        width -= _d;
    }
    /* Draw */
    let mut sy = osy;
    while dy < height {
        let mut dx = odx;
        let mut sx = osx;
        while dx < width {
            blend_pixel(
                &b.mode,
                &mut b.pixels[((x + dx) + (y + dy) * b.w) as usize],
                src.pixels[((sub.x + (sx >> FX_BITS_12)) + (sub.y + (sy >> FX_BITS_12)) * src.w)
                               as usize],
            );
            sx += ix;
            dx += 1;
        }
        sy += iy;
        dy += 1;
    }
}

fn scan_line(
    b: &mut Buffer,
    src: &Buffer,
    sub: &Rect,
    dy: i32,
    t: Transform,
    sx_incr: i32,
    sy_incr: i32,
) {
    /* Adjust for clipping */
    if dy < b.clip.y || dy >= b.clip.y + b.clip.h {
        return;
    }
    let mut left = t.ox as i32;
    let mut right = t.oy as i32;
    let mut sx = t.sx as i32;
    let mut sy = t.sy as i32;
    let _d = b.clip.x - left;
    if _d > 0 {
        left += _d;
        sx += _d * sx_incr;
        sy += _d * sy_incr;
    }
    let _d = right - (b.clip.x + b.clip.w);
    if _d > 0 {
        right -= _d;
    }
    let (mut dx, mut x, mut y);
    /* Does the scaline length go out of bounds of our `sub` rect? If so we
     * should adjust the scan line and the source coordinates accordingly */
    'checkSourceLeft: loop {
        x = sx >> FX_BITS_12;
        y = sy >> FX_BITS_12;
        if x < sub.x || y < sub.y || x >= sub.x + sub.w || y >= sub.y + sub.h {
            left += 1;
            sx += sx_incr;
            sy += sy_incr;
            if left >= right {
                return;
            }
        } else {
            break 'checkSourceLeft;
        }
    }
    'checkSourceRight: loop {
        x = (sx + sx_incr * (right - left)) >> FX_BITS_12;
        y = (sy + sy_incr * (right - left)) >> FX_BITS_12;
        if x < sub.x || y < sub.y || x >= sub.x + sub.w || y >= sub.y + sub.h {
            right -= 1;
            if left >= right {
                return;
            }
        } else {
            break 'checkSourceRight;
        }
    }
    /* Draw */
    dx = left;
    while dx < right {
        blend_pixel(
            &b.mode,
            &mut b.pixels[(dx + dy * b.w) as usize],
            src.pixels[((sx >> FX_BITS_12) + (sy >> FX_BITS_12) * src.w) as usize],
        );
        sx += sx_incr;
        sy += sy_incr;
        dx += 1;
    }
}

pub fn rotate_scaled(b: &mut Buffer, src: &Buffer, x: i32, y: i32, sub: Rect, t: Transform) {
    let mut points: [Point; 4] = [
        Point::new(0, 0),
        Point::new(0, 0),
        Point::new(0, 0),
        Point::new(0, 0),
    ];
    let cosr = t.r.cos();
    let sinr = t.r.sin();
    let abs_sx =
        if t.sx < 0f32 { -t.sx } else { t.sx };
    let abs_sy =
        if t.sy < 0f32 { -t.sy } else { t.sy };
    let inv_x = t.sx < 0f32;
    let inv_y = t.sy < 0f32;
    let width = (sub.w as f32 * abs_sx) as i32;
    let height = (sub.h as f32 * abs_sy) as i32;
    let _q = (t.r * 4f32 / PI2) as i32;
    let cosq = (_q as f32 * PI2 / 4f32).cos();
    let sinq = (_q as f32 * PI2 / 4f32).sin();
    let ox = (if inv_x {
        sub.w as f32 - t.ox
    } else {
        t.ox
    }) * abs_sx;
    let oy = (if inv_y {
        sub.h as f32 - t.oy
    } else {
        t.oy
    }) * abs_sy;
    /* Store rotated corners as points */
    points[0].x = x + (cosr * (-ox) - sinr * (-oy)) as i32;
    points[0].y = y + (sinr * (-ox) + cosr * (-oy)) as i32;
    points[1].x = x + (cosr * (-ox + width as f32) - sinr * (-oy)) as i32;
    points[1].y = y + (sinr * (-ox + width as f32) + cosr * (-oy)) as i32;
    points[2].x = x + (cosr * (-ox + width as f32) - sinr * (-oy + height as f32)) as i32;
    points[2].y = y + (sinr * (-ox + width as f32) + cosr * (-oy + height as f32)) as i32;
    points[3].x = x + (cosr * (-ox) - sinr * (-oy + height as f32)) as i32;
    points[3].y = y + (sinr * (-ox) + cosr * (-oy + height as f32)) as i32;
    /* Set named points based on rotation */
    let top = &points[((-_q) & 3) as usize];
    let right = &points[((-_q + 1) & 3) as usize];
    let bottom = &points[((-_q + 2) & 3) as usize];
    let left = &points[((-_q + 3) & 3) as usize];
    /* Clipped completely off screen? */
    if bottom.y < b.clip.y || top.y >= b.clip.y + b.clip.h {
        return;
    }
    if right.x < b.clip.x || left.x >= b.clip.x + b.clip.w {
        return;
    }
    /* Destination */
    let mut xr = top.x << FX_BITS_12;
    let mut xl = xr;
    let mut il = xdiv_i32((left.x - top.x) << FX_BITS_12, left.y - top.y);
    let mut ir = xdiv_i32((right.x - top.x) << FX_BITS_12, right.y - top.y);
    /* Source */
    let sxi = (xdiv_i32(sub.w << FX_BITS_12, width) as f32 * (-t.r).cos()) as i32;
    let syi = (xdiv_i32(sub.h << FX_BITS_12, height) as f32 * (-t.r).sin()) as i32;
    let mut sxoi = (xdiv_i32(sub.w << FX_BITS_12, left.y - top.y) as f32 * sinq) as i32;
    let mut syoi = (xdiv_i32(sub.h << FX_BITS_12, left.y - top.y) as f32 * cosq) as i32;
    let (mut sx, mut sy) = match _q {
        1 => (sub.x << FX_BITS_12, ((sub.y + sub.h) << FX_BITS_12) - 1),
        2 => (
            ((sub.x + sub.w) << FX_BITS_12) - 1,
            ((sub.y + sub.h) << FX_BITS_12) - 1,
        ),
        3 => (((sub.x + sub.w) << FX_BITS_12) - 1, sub.y << FX_BITS_12),
        _ => (sub.x << FX_BITS_12, sub.y << FX_BITS_12),
    };
    /* Draw */
    let mut dy = if left.y == top.y || right.y == top.y {
        /* Adjust for right-angled rotation */
        top.y - 1
    } else {
        top.y
    };
    while dy <= bottom.y {
        /* Invert source iterators & increments if we are scaled negatively */
        let (tsx, tsxi) = if inv_x {
            (((sub.x * 2 + sub.w) << FX_BITS_12) - sx - 1, -sxi)
        } else {
            (sx, sxi)
        };
        let (tsy, tsyi) = if inv_y {
            (((sub.y * 2 + sub.h) << FX_BITS_12) - sy - 1, -syi)
        } else {
            (sy, syi)
        };
        /* Draw row */
        scan_line(
            b,
            src,
            &sub,
            dy,
            Transform::new(
                (xl >> FX_BITS_12) as f32,
                (xr >> FX_BITS_12) as f32,
                0f32,
                tsx as f32,
                tsy as f32,
            ),
            tsxi,
            tsyi,
        );
        sx += sxoi;
        sy += syoi;
        xl += il;
        xr += ir;
        dy += 1;
        /* Modify increments if we've reached the left or right corner */
        if dy == left.y {
            il = xdiv_i32((bottom.x - left.x) << FX_BITS_12, bottom.y - left.y);
            sxoi = (xdiv_i32(sub.w << FX_BITS_12, bottom.y - left.y) as f32 * cosq) as i32;
            syoi = (xdiv_i32(sub.h << FX_BITS_12, bottom.y - left.y) as f32 * -sinq) as i32;
        }
        if dy == right.y {
            ir = xdiv_i32((bottom.x - right.x) << FX_BITS_12, bottom.y - right.y);
        }
    }
}
