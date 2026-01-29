# ASCII Rendering - Shape Vector Technique

Source code and documentation for the blog post **"ASCII characters are not pixels: a deep dive into ASCII rendering"** by Alex Harri.

This repository contains a production-grade ASCII rendering system that uses **shape vectors** instead of pixel-based character matching, resulting in sharper edges and better visual quality.

## Key Concepts

- **Shape Vectors**: 6-dimensional vectors representing character shapes via sampling circles
- **K-d Tree Lookup**: Fast nearest-neighbor search for character matching
- **GPU Acceleration**: WebGL2 shaders for real-time sampling
- **Contrast Enhancement**: Global and directional crunching for sharp edges

## Repository Structure

```
posts/
  ascii-rendering.md          # Full blog post with interactive demos

src/components/AsciiScene/    # Main ASCII rendering implementation
  alphabets/                  # Pre-computed character shape vectors (JSON)
  characterLookup/            # K-d tree and character matching
  sampling/
    cpu/                      # CPU-based sampling
    gpu/                      # WebGL2 GPU-accelerated sampling
  render/                     # Canvas rendering components

src/components/
  CharacterPlot/              # 2D visualization of shape vector space
  Vector6D/                   # 6D shape vector visualization
  Scene2D/                    # 2D canvas demo scenes

src/threejs/                  # Three.js 3D scene rendering
  scenes/cube.tsx             # 3D cube demo for ASCII rendering

scripts/ascii/                # Alphabet generation scripts
  configs/                    # Sampling configurations
  alphabets/                  # Character set definitions

public/
  fonts/monolisa/             # Monospace font for character rendering
  images/posts/ascii-rendering/ # Blog post assets
```

## Running the Demos

```bash
npm install
npm run dev
```

Then visit `http://localhost:8080/blog/ascii-rendering` to view the interactive blog post.

## Generate Alphabets

To regenerate the character shape vector JSON files:

```bash
npm run generate-alphabets
```

## Core Algorithm

1. **Sampling**: For each grid cell, sample lightness values at 6 circular regions (using Vogel's method for sample distribution)
2. **Shape Vector**: Combine samples into a 6D vector representing the cell's visual pattern
3. **Character Matching**: Find the character whose pre-computed shape vector is closest (Euclidean distance) using a k-d tree
4. **Contrast Enhancement**: Apply global and directional normalization for sharper boundaries

## Notes on Building a Library

### Browser Compatibility
The current implementation uses WebGL 2. A production library would likely need to convert the shaders to WebGL 1 for broader browser support. The API design for such a library is an open question.

### Font Dependency
Shape vectors are **font-specific**. The pre-computed vectors in this repo are for the MonoLisa font. Using a different font requires:

1. Regenerating all character shape vectors for that font
2. Potentially repositioning the sampling circles (internal and external) - different fonts have different glyph proportions and may need adjusted sampling configurations

The `scripts/ascii/` directory contains the generation tooling, but it currently has a hardcoded font path. A proper library would need user-facing tooling to generate shape vectors for custom fonts.

## Credits

Original implementation by [Alex Harri](https://alexharri.com). See the [original repository](https://github.com/alexharri/website).

## License

See [LICENSE.md](LICENSE.md)
