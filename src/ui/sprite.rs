use std::collections::HashMap;

/// UV rectangle within the atlas texture, in normalized 0..1 coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteRect {
    pub u0: f32, // left
    pub v0: f32, // top
    pub u1: f32, // right
    pub v1: f32, // bottom
}

/// Pixel-space rectangle within the atlas RGBA buffer.
#[derive(Debug, Clone, Copy)]
struct PackedRegion {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

/// Shelf row for shelf-packing algorithm.
struct Shelf {
    y: u32,      // top Y of this shelf
    height: u32, // tallest sprite in this shelf
    cursor: u32, // next free X in this shelf
}

/// CPU-side sprite atlas that shelf-packs named RGBA regions.
///
/// Pure data structure — no GPU resources. The renderer (UI-202b)
/// uploads `pixels()` to a wgpu texture.
pub struct SpriteAtlas {
    width: u32,
    height: u32,
    pixels: Vec<u8>, // RGBA, row-major, 4 bytes per pixel
    regions: HashMap<String, PackedRegion>,
    shelves: Vec<Shelf>,
}

impl SpriteAtlas {
    /// Create an empty atlas of the given dimensions.
    /// Minimum 512x512 enforced.
    pub fn new(width: u32, height: u32) -> Self {
        let width = width.max(512);
        let height = height.max(512);
        Self {
            width,
            height,
            pixels: vec![0u8; (width * height * 4) as usize],
            regions: HashMap::new(),
            shelves: Vec::new(),
        }
    }

    /// Atlas width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Atlas height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGBA pixel data, row-major, 4 bytes per pixel.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Look up a named sprite region's UV rect.
    pub fn get(&self, name: &str) -> Option<SpriteRect> {
        self.regions.get(name).map(|r| SpriteRect {
            u0: r.x as f32 / self.width as f32,
            v0: r.y as f32 / self.height as f32,
            u1: (r.x + r.w) as f32 / self.width as f32,
            v1: (r.y + r.h) as f32 / self.height as f32,
        })
    }

    /// Register a named region by blitting RGBA pixel data into the atlas.
    /// Returns the UV rect on success, or None if the region doesn't fit.
    ///
    /// `data` must be `w * h * 4` bytes (RGBA, row-major).
    pub fn pack(&mut self, name: &str, w: u32, h: u32, data: &[u8]) -> Option<SpriteRect> {
        if data.len() != (w * h * 4) as usize {
            return None;
        }
        if w == 0 || h == 0 {
            return None;
        }
        if self.regions.contains_key(name) {
            return self.get(name);
        }

        // Try to fit in an existing shelf.
        let mut placed = None;
        for shelf in &mut self.shelves {
            if h <= shelf.height && shelf.cursor + w <= self.width {
                placed = Some((shelf.cursor, shelf.y));
                shelf.cursor += w;
                break;
            }
        }

        // Open a new shelf if needed.
        if placed.is_none() {
            let shelf_y = self.shelves.last().map(|s| s.y + s.height).unwrap_or(0);
            if shelf_y + h > self.height {
                return None; // atlas full
            }
            self.shelves.push(Shelf {
                y: shelf_y,
                height: h,
                cursor: w,
            });
            placed = Some((0, shelf_y));
        }

        let (px, py) = placed?;

        // Blit pixel data into the atlas.
        for row in 0..h {
            let src_start = (row * w * 4) as usize;
            let src_end = src_start + (w * 4) as usize;
            let dst_start = ((py + row) * self.width * 4 + px * 4) as usize;
            let dst_end = dst_start + (w * 4) as usize;
            self.pixels[dst_start..dst_end].copy_from_slice(&data[src_start..src_end]);
        }

        let region = PackedRegion { x: px, y: py, w, h };
        self.regions.insert(name.to_string(), region);
        self.get(name)
    }

    /// Load a PNG file and register all icon regions from a manifest.
    ///
    /// The manifest maps `name -> (x, y, w, h)` pixel rects within the PNG.
    /// Each region is blitted into the atlas via shelf-packing.
    pub fn load_png_with_manifest(
        &mut self,
        png_data: &[u8],
        manifest: &[(String, u32, u32, u32, u32)],
    ) -> Vec<(String, Option<SpriteRect>)> {
        let img = match image::load_from_memory_with_format(png_data, image::ImageFormat::Png) {
            Ok(img) => img.into_rgba8(),
            Err(_) => {
                return manifest
                    .iter()
                    .map(|(name, ..)| (name.clone(), None))
                    .collect();
            }
        };

        let img_w = img.width();
        let mut results = Vec::new();

        for (name, sx, sy, sw, sh) in manifest {
            // Extract sub-region from the source PNG.
            let mut region_data = vec![0u8; (*sw * *sh * 4) as usize];
            for row in 0..*sh {
                let src_y = sy + row;
                let src_start = ((src_y * img_w + sx) * 4) as usize;
                let dst_start = (row * sw * 4) as usize;
                let len = (*sw * 4) as usize;
                if src_start + len <= img.as_raw().len() {
                    region_data[dst_start..dst_start + len]
                        .copy_from_slice(&img.as_raw()[src_start..src_start + len]);
                }
            }

            let uv = self.pack(name, *sw, *sh, &region_data);
            results.push((name.clone(), uv));
        }

        results
    }

    /// Number of registered sprite regions.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_three_regions_non_overlapping() {
        let mut atlas = SpriteAtlas::new(512, 512);

        // Pack three 16x16 icons.
        let red = vec![255u8; 16 * 16 * 4];
        let green = vec![128u8; 16 * 16 * 4];
        let blue = vec![64u8; 16 * 16 * 4];

        let r1 = atlas.pack("heart", 16, 16, &red);
        let r2 = atlas.pack("sword", 16, 16, &green);
        let r3 = atlas.pack("shield", 16, 16, &blue);

        assert!(r1.is_some());
        assert!(r2.is_some());
        assert!(r3.is_some());

        let r1 = r1.expect("heart");
        let r2 = r2.expect("sword");
        let r3 = r3.expect("shield");

        // Verify non-overlapping UV rects.
        assert!(!rects_overlap(r1, r2), "heart and sword should not overlap");
        assert!(
            !rects_overlap(r1, r3),
            "heart and shield should not overlap"
        );
        assert!(
            !rects_overlap(r2, r3),
            "sword and shield should not overlap"
        );

        assert_eq!(atlas.region_count(), 3);
    }

    #[test]
    fn uv_rects_correct_for_known_size() {
        let mut atlas = SpriteAtlas::new(512, 512);
        let data = vec![255u8; 32 * 16 * 4]; // 32 wide, 16 tall
        let uv = atlas.pack("wide", 32, 16, &data).expect("should fit");

        // First packed region starts at (0, 0).
        assert!((uv.u0 - 0.0).abs() < 0.001);
        assert!((uv.v0 - 0.0).abs() < 0.001);
        assert!((uv.u1 - 32.0 / 512.0).abs() < 0.001);
        assert!((uv.v1 - 16.0 / 512.0).abs() < 0.001);
    }

    #[test]
    fn duplicate_name_returns_existing() {
        let mut atlas = SpriteAtlas::new(512, 512);
        let data = vec![255u8; 16 * 16 * 4];
        let r1 = atlas.pack("icon", 16, 16, &data);
        let r2 = atlas.pack("icon", 16, 16, &data);
        assert_eq!(r1, r2);
        assert_eq!(atlas.region_count(), 1);
    }

    #[test]
    fn atlas_full_returns_none() {
        // Tiny atlas: 32x32 = 1024 pixels = room for 4 x 16x16 icons.
        let mut atlas = SpriteAtlas::new(512, 512);
        // But shelf-packing may waste some space. Fill with many icons.
        let data = vec![255u8; 256 * 256 * 4];
        let r1 = atlas.pack("big1", 256, 256, &data);
        let r2 = atlas.pack("big2", 256, 256, &data);
        let r3 = atlas.pack("big3", 256, 256, &data);
        let r4 = atlas.pack("big4", 256, 256, &data);
        assert!(r1.is_some());
        assert!(r2.is_some());
        assert!(r3.is_some());
        assert!(r4.is_some());
        // 5th 256x256 won't fit in 512x512.
        let r5 = atlas.pack("big5", 256, 256, &data);
        assert!(r5.is_none(), "atlas should be full");
    }

    #[test]
    fn zero_size_rejected() {
        let mut atlas = SpriteAtlas::new(512, 512);
        assert!(atlas.pack("empty", 0, 0, &[]).is_none());
        assert!(atlas.pack("zero_w", 0, 16, &[]).is_none());
    }

    #[test]
    fn wrong_data_length_rejected() {
        let mut atlas = SpriteAtlas::new(512, 512);
        let data = vec![255u8; 10]; // wrong size for 16x16
        assert!(atlas.pack("bad", 16, 16, &data).is_none());
    }

    #[test]
    fn pixel_data_blitted_correctly() {
        let mut atlas = SpriteAtlas::new(512, 512);
        // 2x2 icon with known pixel values.
        let data: Vec<u8> = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 0, 255, // (1,1) yellow
        ];
        atlas.pack("test", 2, 2, &data).expect("should fit");

        // Check pixel at atlas (0,0) = red.
        assert_eq!(atlas.pixels[0], 255);
        assert_eq!(atlas.pixels[1], 0);
        assert_eq!(atlas.pixels[2], 0);
        assert_eq!(atlas.pixels[3], 255);

        // Check pixel at atlas (1,0) = green.
        assert_eq!(atlas.pixels[4], 0);
        assert_eq!(atlas.pixels[5], 255);
        assert_eq!(atlas.pixels[6], 0);

        // Check pixel at atlas (0,1) = blue (row 1, col 0).
        let row1_offset = (512 * 4) as usize; // row 1 start
        assert_eq!(atlas.pixels[row1_offset], 0);
        assert_eq!(atlas.pixels[row1_offset + 1], 0);
        assert_eq!(atlas.pixels[row1_offset + 2], 255);
    }

    #[test]
    fn shelf_packing_opens_new_shelf_for_tall_sprite() {
        let mut atlas = SpriteAtlas::new(512, 512);
        // Pack a short sprite then a tall one.
        let short = vec![255u8; 64 * 16 * 4]; // 64w x 16h
        let tall = vec![128u8; 32 * 64 * 4]; // 32w x 64h

        atlas.pack("short", 64, 16, &short).expect("should fit");
        let uv = atlas.pack("tall", 32, 64, &tall).expect("should fit");

        // Tall sprite should be on a new shelf (y=16, since first shelf is 16px tall).
        assert!(
            (uv.v0 - 16.0 / 512.0).abs() < 0.001,
            "tall sprite should start at y=16"
        );
    }

    /// Helper: check if two UV rects overlap.
    fn rects_overlap(a: SpriteRect, b: SpriteRect) -> bool {
        a.u0 < b.u1 && a.u1 > b.u0 && a.v0 < b.v1 && a.v1 > b.v0
    }
}
