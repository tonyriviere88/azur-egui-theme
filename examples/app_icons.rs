//! Builds the executable icons in `app-icons/` from the marks in
//! `app-icons/marks.rs`.
//!
//!     cargo run --example app_icons
//!
//! Writes, per application: `app-16.png` … `app-256.png`, an `app.ico` holding all
//! nine, and one `app-icons/preview.png` sheet to look at.
//!
//! There is no egui here. The marks are geometry (`marks::Prim`), so they can be
//! rasterised directly — which keeps this deterministic, keeps it off the GPU, and
//! means the same definitions drive the painted title-bar icon and these pixels.

// One copy of the geometry, shared by both media. The generator reads the
// primitives; the `Icon` functions beside them are for the application to call, so
// they are dead code *here* and nowhere else.
#[path = "../app-icons/marks.rs"]
#[allow(dead_code)]
mod marks;

use image::{Rgba, RgbaImage};
use marks::{Mark, Prim};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// The two colors, as asked for: the dark theme's `background-layer` and its
/// `accent.mark`. Both are Azur palette values — `GRAY_2` and `AZURE_70` — so the
/// desktop icon is in the same paint as the window it opens.
const PLATE: [u8; 3] = [0x14, 0x17, 0x1a];
const MARK: [u8; 3] = [0x3a, 0xa0, 0xff];

/// Every size Windows asks for: 16/32/48/256 for its own views, and 20/24/40/64/128
/// for the display-scaling steps in between. `docs/icons.md` §13.
const SIZES: [u32; 9] = [16, 20, 24, 32, 40, 48, 64, 128, 256];

/// The plate's corner radius as a fraction of the side — `radius-large` on a 32px
/// control, which is the corner family the app's own cards use. §14.
const CORNER: f32 = 0.25;

/// Subsamples per pixel per axis. 4×4 is enough for a 16px icon to come out clean
/// and cheap enough that all 36 images render instantly.
const SS: u32 = 4;

fn main() -> std::io::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app-icons");
    let mut previews = Vec::new();

    for mark in marks::ALL {
        let dir = root.join(mark.slug);
        std::fs::create_dir_all(&dir)?;

        let rendered: Vec<(u32, RgbaImage)> = SIZES.iter().map(|&s| (s, render(mark, s))).collect();

        for (size, image) in &rendered {
            let path = dir.join(format!("app-{size}.png"));
            image.save(&path).map_err(to_io)?;
        }
        write_ico(&dir.join("app.ico"), &rendered)?;
        println!("{:<18} {} pngs + app.ico", mark.slug, rendered.len());
        previews.push(rendered);
    }

    let sheet = preview(&previews);
    let path = root.join("preview.png");
    sheet.save(&path).map_err(to_io)?;
    println!("preview            {}", path.display());
    Ok(())
}

fn to_io(e: image::ImageError) -> std::io::Error {
    std::io::Error::other(e)
}

// ---------------------------------------------------------------------------
// Rasterising a mark
// ---------------------------------------------------------------------------

/// One icon: the plate, with the mark on it, alpha only in the plate's corners.
fn render(mark: &Mark, size: u32) -> RgbaImage {
    let scale = size as f32 / 16.0;
    // A 16px mark stroked at 1.4 units would be 1.4px and read as gray. Below 32px
    // the weight is floored instead — `docs/icons.md` §13: the small sizes are
    // pixel work, not a downscale.
    let stroke = (mark.stroke * scale).max(if size <= 32 { 2.0 } else { 0.0 });
    // Whole-pixel edges are only worth having if the weight is a whole number too.
    let (stroke, snap) = if size <= 48 {
        (stroke.round().max(1.0), true)
    } else {
        (stroke, false)
    };
    let radius = size as f32 * CORNER;
    let half = size as f32 * 0.5;

    let mut out = RgbaImage::new(size, size);
    let step = 1.0 / SS as f32;
    let total = (SS * SS) as f32;

    for py in 0..size {
        for px in 0..size {
            let (mut plate_hits, mut mark_hits) = (0.0f32, 0.0f32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let x = px as f32 + (sx as f32 + 0.5) * step;
                    let y = py as f32 + (sy as f32 + 0.5) * step;
                    if round_box(x, y, half, half, half, half, radius) <= 0.0 {
                        plate_hits += 1.0;
                    }
                    if inside_mark(mark, x, y, scale, stroke, snap) {
                        mark_hits += 1.0;
                    }
                }
            }
            let plate = plate_hits / total;
            let ink = (mark_hits / total).min(plate);
            let rgb = mix(PLATE, MARK, ink);
            out.put_pixel(
                px,
                py,
                Rgba([rgb[0], rgb[1], rgb[2], (plate * 255.0).round() as u8]),
            );
        }
    }
    out
}

/// Is this device-space point inside the mark's ink?
///
/// `snap` puts the edges of anything axis-aligned on whole pixels, which is the
/// difference between a crisp 16px icon and a gray one: a 2px bar centred on x=4.7
/// covers two pixels at 65% and 35% and reads as a smudge. Diagonals — the
/// folder's tab — cannot be snapped and do not need to be.
fn inside_mark(mark: &Mark, x: f32, y: f32, scale: f32, stroke: f32, snap: bool) -> bool {
    let half = stroke * 0.5;
    // Grid units -> device pixels.
    let g = |u: f32| u * scale;
    // A stroke of width `w` centred on `c` covers `c ± w/2`; putting the near edge
    // on an integer is what makes it land on whole pixels.
    let center = |v: f32| if snap { (v - half).round() + half } else { v };
    let edge = |v: f32| if snap { v.round() } else { v };

    mark.prims.iter().any(|prim| match *prim {
        Prim::Path(points) => polyline(points, x, y, scale, false) <= half,
        Prim::Closed(points) => polyline(points, x, y, scale, true) <= half,
        Prim::Line(x1, y1, x2, y2) => {
            let (mut ax, mut ay, mut bx, mut by) = (g(x1), g(y1), g(x2), g(y2));
            if (ax - bx).abs() < 0.01 {
                ax = center(ax);
                bx = ax;
            }
            if (ay - by).abs() < 0.01 {
                ay = center(ay);
                by = ay;
            }
            segment(x, y, ax, ay, bx, by) <= half
        }
        Prim::Rect {
            x: rx,
            y: ry,
            w,
            h,
            r,
            fill,
        } => {
            let (x0, y0) = (edge(g(rx)), edge(g(ry)));
            let (x1, y1) = (edge(g(rx + w)), edge(g(ry + h)));
            let d = round_box(
                x,
                y,
                (x0 + x1) * 0.5,
                (y0 + y1) * 0.5,
                (x1 - x0) * 0.5,
                (y1 - y0) * 0.5,
                g(r),
            );
            if fill {
                d <= 0.0
            } else {
                d.abs() <= half
            }
        }
    })
}

/// Distance from a point to a polyline, in device space.
fn polyline(points: &[(f32, f32)], x: f32, y: f32, scale: f32, closed: bool) -> f32 {
    let mut best = f32::MAX;
    let n = points.len();
    let last = if closed { n } else { n - 1 };
    for i in 0..last {
        let (ax, ay) = points[i];
        let (bx, by) = points[(i + 1) % n];
        best = best.min(segment(
            x,
            y,
            ax * scale,
            ay * scale,
            bx * scale,
            by * scale,
        ));
    }
    best
}

/// Distance from `p` to the segment `a`–`b`.
fn segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (pax, pay) = (px - ax, py - ay);
    let (bax, bay) = (bx - ax, by - ay);
    let len2 = bax * bax + bay * bay;
    let h = if len2 > 0.0 {
        ((pax * bax + pay * bay) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (pax - bax * h, pay - bay * h);
    (dx * dx + dy * dy).sqrt()
}

/// Signed distance to a rounded box: negative inside, zero on the edge.
fn round_box(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let r = r.min(hw).min(hh);
    let qx = (px - cx).abs() - (hw - r);
    let qy = (py - cy).abs() - (hh - r);
    let (ox, oy) = (qx.max(0.0), qy.max(0.0));
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

/// Mix two sRGB colors through linear light, so an antialiased edge does not come
/// out darker than the two sides it sits between.
fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let to_linear = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    let to_srgb = |l: f32| {
        let s = if l <= 0.0031308 {
            l * 12.92
        } else {
            1.055 * l.powf(1.0 / 2.4) - 0.055
        };
        (s.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    let mut out = [0u8; 3];
    for i in 0..3 {
        out[i] = to_srgb(to_linear(a[i]) + (to_linear(b[i]) - to_linear(a[i])) * t);
    }
    out
}

// ---------------------------------------------------------------------------
// The .ico container
// ---------------------------------------------------------------------------

/// Write a multi-size `.ico` with a PNG payload per entry.
///
/// The format is a 6-byte header, a 16-byte directory entry per image, then the
/// payloads. PNG-compressed entries have been read by Windows since Vista, which
/// saves carrying a second (BMP, bottom-up, with an AND mask) encoder for the
/// small sizes.
fn write_ico(path: &Path, images: &[(u32, RgbaImage)]) -> std::io::Result<()> {
    let payloads: Vec<Vec<u8>> = images
        .iter()
        .map(|(_, image)| {
            let mut buf = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
                .expect("in-memory PNG encode");
            buf
        })
        .collect();

    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // 1 = icon
    out.extend_from_slice(&(images.len() as u16).to_le_bytes());

    let mut offset = 6 + 16 * images.len() as u32;
    for ((size, _), png) in images.iter().zip(&payloads) {
        // 256 does not fit in a byte and is written as 0.
        let dim = if *size >= 256 { 0u8 } else { *size as u8 };
        out.push(dim); // width
        out.push(dim); // height
        out.push(0); // palette size, 0 for true color
        out.push(0); // reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // color planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for png in &payloads {
        out.extend_from_slice(png);
    }
    std::fs::write(path, out)
}

// ---------------------------------------------------------------------------
// The preview sheet
// ---------------------------------------------------------------------------

/// One image showing every icon at every size, on both a dark and a light ground,
/// plus the two small sizes magnified so their pixels can be counted.
///
/// Columns are the four applications in `marks::ALL` order: task manager, file
/// explorer, audio mixer, program launcher.
fn preview(apps: &[Vec<(u32, RgbaImage)>]) -> RgbaImage {
    const PAD: u32 = 20;
    const COL: u32 = 256;
    let width = PAD + (COL + PAD) * apps.len() as u32;

    let small: Vec<u32> = SIZES.iter().copied().filter(|&s| s <= 64).collect();
    let row_small = 64;
    // 7x for the 16 and 4x for the 32: integer multiples, so nearest-neighbour
    // magnification shows square pixels rather than a moire of 7s and 8s.
    let (zoom16, zoom32) = (16 * 7, 32 * 4);
    let zoom = zoom16.max(zoom32);
    // 16+20+24+32+40+48+64 = 244, so a 2px gap is what a 256 column has room for.
    const GAP: u32 = 2;
    let height = PAD + 256 + PAD + 128 + PAD + row_small + PAD + row_small + PAD + zoom + PAD;

    let dark = Rgba([0x07, 0x08, 0x09, 0xff]);
    let light = Rgba([0xf5, 0xf6, 0xf8, 0xff]);
    let mut sheet = RgbaImage::from_pixel(width, height, dark);

    // The band the light row sits on, so the same icons can be judged against a
    // light taskbar without a second file.
    let light_top = PAD + 256 + PAD + 128 + PAD + row_small + PAD;
    for y in light_top - PAD / 2..light_top + row_small + PAD / 2 {
        for x in 0..width {
            sheet.put_pixel(x, y, light);
        }
    }

    for (col, rendered) in apps.iter().enumerate() {
        let left = PAD + (COL + PAD) * col as u32;
        let at = |size: u32| -> &RgbaImage {
            &rendered
                .iter()
                .find(|(s, _)| *s == size)
                .expect("every size rendered")
                .1
        };

        let mut y = PAD;
        blit(&mut sheet, at(256), left, y);
        y += 256 + PAD;
        blit(&mut sheet, at(128), left, y);
        y += 128 + PAD;

        // The small sizes in a row, bottom-aligned, on dark and then on light.
        for top in [y, light_top] {
            let mut x = left;
            for &size in &small {
                blit(&mut sheet, at(size), x, top + (row_small - size));
                x += size + GAP;
            }
        }
        y = light_top + row_small + PAD;

        // 16px at 8× and 32px at 4×, nearest neighbour: what the pixels actually are.
        let nearest = image::imageops::FilterType::Nearest;
        let z16 = image::imageops::resize(at(16), zoom16, zoom16, nearest);
        let z32 = image::imageops::resize(at(32), zoom32, zoom32, nearest);
        blit(&mut sheet, &z16, left, y + (zoom - zoom16));
        blit(&mut sheet, &z32, left + zoom16 + 8, y + (zoom - zoom32));
    }
    sheet
}

/// Alpha-composite `src` onto `dst` at `(ox, oy)`.
fn blit(dst: &mut RgbaImage, src: &RgbaImage, ox: u32, oy: u32) {
    for (x, y, px) in src.enumerate_pixels() {
        let (dx, dy) = (ox + x, oy + y);
        if dx >= dst.width() || dy >= dst.height() {
            continue;
        }
        let a = px[3] as f32 / 255.0;
        let under = *dst.get_pixel(dx, dy);
        let mut blended = [0u8; 4];
        for i in 0..3 {
            blended[i] = mix([under[i]; 3], [px[i]; 3], a)[0];
        }
        blended[3] = 0xff;
        dst.put_pixel(dx, dy, Rgba(blended));
    }
}
