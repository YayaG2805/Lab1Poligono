use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 450;

pub const BACKGROUND: Color = Color::new(245, 245, 245, 255);
pub const POLYGON_1_FILL: Color = Color::new(255, 155, 155, 255);
pub const POLYGON_1_OUTLINE: Color = Color::new(150, 24, 24, 255);
pub const POLYGON_2_FILL: Color = Color::new(145, 218, 165, 255);
pub const POLYGON_2_OUTLINE: Color = Color::new(24, 112, 51, 255);
pub const POLYGON_3_FILL: Color = Color::new(145, 186, 255, 255);
pub const POLYGON_3_OUTLINE: Color = Color::new(22, 63, 145, 255);
pub const POLYGON_4_FILL: Color = Color::new(255, 218, 124, 255);
pub const POLYGON_4_OUTLINE: Color = Color::new(154, 86, 12, 255);
pub const HOLE_OUTLINE: Color = Color::new(64, 38, 8, 255);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug)]
pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, background: Color) -> Self {
        Self {
            width,
            height,
            pixels: vec![background; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }

        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            return;
        }

        self.pixels[y * self.width + x] = color;
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        if x < 0 || y < 0 {
            return None;
        }

        let (x, y) = (x as usize, y as usize);
        (x < self.width && y < self.height).then(|| self.pixels[y * self.width + x])
    }

    pub fn as_rgba8(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 4);
        for pixel in &self.pixels {
            bytes.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
        }
        bytes
    }
}

pub const POLYGON_1: [Point; 10] = [
    Point::new(165, 380),
    Point::new(185, 360),
    Point::new(180, 330),
    Point::new(207, 345),
    Point::new(233, 330),
    Point::new(230, 360),
    Point::new(250, 380),
    Point::new(220, 385),
    Point::new(205, 410),
    Point::new(193, 383),
];

pub const POLYGON_2: [Point; 4] = [
    Point::new(321, 335),
    Point::new(288, 286),
    Point::new(339, 251),
    Point::new(374, 302),
];

pub const POLYGON_3: [Point; 3] = [
    Point::new(377, 249),
    Point::new(411, 197),
    Point::new(436, 249),
];

pub const POLYGON_4: [Point; 18] = [
    Point::new(413, 177),
    Point::new(448, 159),
    Point::new(502, 88),
    Point::new(553, 53),
    Point::new(535, 36),
    Point::new(676, 37),
    Point::new(660, 52),
    Point::new(750, 145),
    Point::new(761, 179),
    Point::new(672, 192),
    Point::new(659, 214),
    Point::new(615, 214),
    Point::new(632, 230),
    Point::new(580, 230),
    Point::new(597, 215),
    Point::new(552, 214),
    Point::new(517, 144),
    Point::new(466, 180),
];

pub const POLYGON_5_HOLE: [Point; 4] = [
    Point::new(682, 175),
    Point::new(708, 120),
    Point::new(735, 148),
    Point::new(739, 170),
];

pub fn render_scene() -> Framebuffer {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, BACKGROUND);

    fill_polygon_scanline(&mut framebuffer, &[&POLYGON_1], POLYGON_1_FILL);
    fill_polygon_scanline(&mut framebuffer, &[&POLYGON_2], POLYGON_2_FILL);
    fill_polygon_scanline(&mut framebuffer, &[&POLYGON_3], POLYGON_3_FILL);
    fill_polygon_scanline(
        &mut framebuffer,
        &[&POLYGON_4, &POLYGON_5_HOLE],
        POLYGON_4_FILL,
    );

    draw_closed_polyline(&mut framebuffer, &POLYGON_1, POLYGON_1_OUTLINE);
    draw_closed_polyline(&mut framebuffer, &POLYGON_2, POLYGON_2_OUTLINE);
    draw_closed_polyline(&mut framebuffer, &POLYGON_3, POLYGON_3_OUTLINE);
    draw_closed_polyline(&mut framebuffer, &POLYGON_4, POLYGON_4_OUTLINE);
    draw_closed_polyline_with_width(&mut framebuffer, &POLYGON_5_HOLE, HOLE_OUTLINE, 3);

    framebuffer
}

pub fn fill_polygon_scanline(
    framebuffer: &mut Framebuffer,
    contours: &[&[Point]],
    fill_color: Color,
) {
    if contours.is_empty() {
        return;
    }

    let Some((min_y, max_y)) = vertical_bounds(contours) else {
        return;
    };

    let start_y = min_y.clamp(0, framebuffer.height() as i32);
    let end_y = max_y.clamp(0, framebuffer.height() as i32);

    for y in start_y..end_y {
        let scan_y = f64::from(y) + 0.5;
        let mut intersections = Vec::new();

        for contour in contours {
            for edge in contour_edges(contour) {
                let (a, b) = edge;
                if a.y == b.y {
                    continue;
                }

                let min_edge_y = a.y.min(b.y) as f64;
                let max_edge_y = a.y.max(b.y) as f64;

                // The interval is semi-open so a vertex shared by two edges is
                // counted once. Horizontal edges do not cross a scanline and
                // are drawn later as outline, so they are skipped here.
                if scan_y >= min_edge_y && scan_y < max_edge_y {
                    let t = (scan_y - f64::from(a.y)) / f64::from(b.y - a.y);
                    let x = f64::from(a.x) + t * f64::from(b.x - a.x);
                    intersections.push(x);
                }
            }
        }

        intersections.sort_by(f64::total_cmp);

        for pair in intersections.chunks_exact(2) {
            let start_x = (pair[0].ceil() as i32).clamp(0, framebuffer.width() as i32);
            let end_x = (pair[1].ceil() as i32).clamp(0, framebuffer.width() as i32);

            for x in start_x..end_x {
                framebuffer.set_pixel(x, y, fill_color);
            }
        }
    }
}

pub fn draw_closed_polyline(framebuffer: &mut Framebuffer, points: &[Point], color: Color) {
    for (start, end) in contour_edges(points) {
        draw_line_bresenham(framebuffer, start, end, color);
    }
}

pub fn draw_closed_polyline_with_width(
    framebuffer: &mut Framebuffer,
    points: &[Point],
    color: Color,
    width: i32,
) {
    let radius = (width.max(1) - 1) / 2;
    for (start, end) in contour_edges(points) {
        for offset_y in -radius..=radius {
            for offset_x in -radius..=radius {
                draw_line_bresenham(
                    framebuffer,
                    Point::new(start.x + offset_x, start.y + offset_y),
                    Point::new(end.x + offset_x, end.y + offset_y),
                    color,
                );
            }
        }
    }
}

pub fn draw_line_bresenham(framebuffer: &mut Framebuffer, start: Point, end: Point, color: Color) {
    let (mut x0, mut y0) = (start.x, start.y);
    let (x1, y1) = (end.x, end.y);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        framebuffer.set_pixel(x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }

        let twice_err = 2 * err;
        if twice_err >= dy {
            err += dy;
            x0 += sx;
        }
        if twice_err <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

pub fn export_outputs(framebuffer: &Framebuffer) -> io::Result<()> {
    write_png(framebuffer, "out.png")?;
    write_bmp(framebuffer, "out.bmp")?;
    Ok(())
}

pub fn write_png(framebuffer: &Framebuffer, path: impl AsRef<Path>) -> io::Result<()> {
    let mut raw = Vec::with_capacity((framebuffer.width * 4 + 1) * framebuffer.height);
    for y in 0..framebuffer.height {
        raw.push(0);
        for x in 0..framebuffer.width {
            let pixel = framebuffer.pixels[y * framebuffer.width + x];
            raw.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
        }
    }

    let mut zlib = Vec::new();
    zlib.extend_from_slice(&[0x78, 0x01]);

    for (index, block) in raw.chunks(65_535).enumerate() {
        let is_last = index == raw.len().div_ceil(65_535) - 1;
        zlib.push(u8::from(is_last));
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }

    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut file = File::create(path)?;
    file.write_all(b"\x89PNG\r\n\x1a\n")?;
    write_png_chunk(&mut file, b"IHDR", &png_ihdr(framebuffer))?;
    write_png_chunk(&mut file, b"IDAT", &zlib)?;
    write_png_chunk(&mut file, b"IEND", &[])?;
    Ok(())
}

pub fn write_bmp(framebuffer: &Framebuffer, path: impl AsRef<Path>) -> io::Result<()> {
    let pixel_bytes = framebuffer.width * framebuffer.height * 4;
    let file_size = 14 + 40 + pixel_bytes;
    let mut file = File::create(path)?;

    file.write_all(b"BM")?;
    file.write_all(&(file_size as u32).to_le_bytes())?;
    file.write_all(&[0; 4])?;
    file.write_all(&(54u32).to_le_bytes())?;

    file.write_all(&(40u32).to_le_bytes())?;
    file.write_all(&(framebuffer.width as i32).to_le_bytes())?;
    file.write_all(&(-(framebuffer.height as i32)).to_le_bytes())?;
    file.write_all(&(1u16).to_le_bytes())?;
    file.write_all(&(32u16).to_le_bytes())?;
    file.write_all(&(0u32).to_le_bytes())?;
    file.write_all(&(pixel_bytes as u32).to_le_bytes())?;
    file.write_all(&(2_835u32).to_le_bytes())?;
    file.write_all(&(2_835u32).to_le_bytes())?;
    file.write_all(&(0u32).to_le_bytes())?;
    file.write_all(&(0u32).to_le_bytes())?;

    for pixel in &framebuffer.pixels {
        file.write_all(&[pixel.b, pixel.g, pixel.r, pixel.a])?;
    }

    Ok(())
}

fn vertical_bounds(contours: &[&[Point]]) -> Option<(i32, i32)> {
    let mut points = contours.iter().flat_map(|contour| contour.iter());
    let first = points.next()?;
    let (mut min_y, mut max_y) = (first.y, first.y);

    for point in points {
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    Some((min_y, max_y))
}

fn contour_edges(points: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
}

fn png_ihdr(framebuffer: &Framebuffer) -> [u8; 13] {
    let mut ihdr = [0; 13];
    ihdr[0..4].copy_from_slice(&(framebuffer.width as u32).to_be_bytes());
    ihdr[4..8].copy_from_slice(&(framebuffer.height as u32).to_be_bytes());
    ihdr[8] = 8;
    ihdr[9] = 6;
    ihdr
}

fn write_png_chunk(file: &mut File, chunk_type: &[u8; 4], data: &[u8]) -> io::Result<()> {
    file.write_all(&(data.len() as u32).to_be_bytes())?;
    file.write_all(chunk_type)?;
    file.write_all(data)?;

    let mut crc_data = Vec::with_capacity(chunk_type.len() + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    file.write_all(&crc32(&crc_data).to_be_bytes())?;
    Ok(())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let (mut a, mut b) = (1u32, 0u32);

    for &byte in bytes {
        a = (a + u32::from(byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn drawing_clips_pixels_outside_framebuffer() {
        let mut framebuffer = Framebuffer::new(4, 4, BACKGROUND);
        let huge_triangle = [
            Point::new(-100, -100),
            Point::new(100, 2),
            Point::new(-100, 100),
        ];

        fill_polygon_scanline(&mut framebuffer, &[&huge_triangle], POLYGON_1_FILL);
        draw_line_bresenham(
            &mut framebuffer,
            Point::new(-50, -50),
            Point::new(50, 50),
            POLYGON_1_OUTLINE,
        );

        assert_eq!(framebuffer.width(), 4);
        assert_eq!(framebuffer.height(), 4);
        assert_eq!(framebuffer.pixels.len(), 16);
    }

    #[test]
    fn known_scene_points_have_expected_colors() {
        let framebuffer = render_scene();

        assert_eq!(framebuffer.get_pixel(205, 370), Some(POLYGON_1_FILL));
        assert_eq!(framebuffer.get_pixel(330, 295), Some(POLYGON_2_FILL));
        assert_eq!(framebuffer.get_pixel(411, 230), Some(POLYGON_3_FILL));
        assert_eq!(framebuffer.get_pixel(560, 100), Some(POLYGON_4_FILL));
        assert_eq!(framebuffer.get_pixel(715, 155), Some(BACKGROUND));
        assert_eq!(framebuffer.get_pixel(20, 20), Some(BACKGROUND));
    }

    #[test]
    fn output_files_are_generated_with_expected_dimensions() {
        let framebuffer = render_scene();
        let temp_dir =
            std::env::temp_dir().join(format!("lab1_poligono_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let png_path = temp_dir.join("out.png");
        let bmp_path = temp_dir.join("out.bmp");
        write_png(&framebuffer, &png_path).unwrap();
        write_bmp(&framebuffer, &bmp_path).unwrap();

        let png = fs::read(&png_path).unwrap();
        let bmp = fs::read(&bmp_path).unwrap();

        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 800);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 450);

        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(i32::from_le_bytes(bmp[18..22].try_into().unwrap()), 800);
        assert_eq!(i32::from_le_bytes(bmp[22..26].try_into().unwrap()), -450);

        fs::remove_file(png_path).unwrap();
        fs::remove_file(bmp_path).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }
}
