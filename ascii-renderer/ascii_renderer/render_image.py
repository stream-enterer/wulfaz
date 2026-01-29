"""
Render ASCII art to an image file.

Outputs monospace ASCII on a grid with grey text on black background.
"""

import argparse
import sys

from PIL import Image, ImageDraw, ImageFont

from .processor import RenderConfig, render_file_to_ascii


def get_monospace_font(size: int = 16) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    """Get a monospace font for rendering."""
    font_names = [
        "DejaVuSansMono.ttf",
        "LiberationMono-Regular.ttf",
        "Consolas.ttf",
        "Courier New.ttf",
        "monospace",
    ]
    for name in font_names:
        try:
            return ImageFont.truetype(name, size)
        except (OSError, IOError):
            continue
    return ImageFont.load_default()


def render_to_image(
    ascii_art: str,
    output_path: str,
    char_width: int = 10,
    char_height: int = 18,
    font_size: int = 16,
    fg_color: tuple[int, int, int] = (180, 180, 180),
    bg_color: tuple[int, int, int] = (0, 0, 0),
) -> None:
    """
    Render ASCII art string to an image file.

    Args:
        ascii_art: The ASCII art string with newlines.
        output_path: Path to save the output image.
        char_width: Width of each character cell in pixels.
        char_height: Height of each character cell in pixels.
        font_size: Font size for rendering.
        fg_color: Foreground (text) color as RGB tuple.
        bg_color: Background color as RGB tuple.
    """
    lines = ascii_art.split('\n')

    img_width = max(len(line) for line in lines) * char_width
    img_height = len(lines) * char_height

    img = Image.new('RGB', (img_width, img_height), bg_color)
    draw = ImageDraw.Draw(img)
    font = get_monospace_font(font_size)

    for row, line in enumerate(lines):
        for col, char in enumerate(line):
            x = col * char_width
            y = row * char_height
            draw.text((x, y), char, fill=fg_color, font=font)

    img.save(output_path)


def parse_args(args: list[str] | None = None) -> argparse.Namespace:
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        prog="ascii-render-image",
        description="Convert images to ASCII art and save as an image file.",
    )

    parser.add_argument("image", help="Path to input image file")
    parser.add_argument("output", help="Path to output image file (PNG, JPG, etc.)")

    parser.add_argument(
        "-w", "--width",
        type=int,
        default=100,
        help="Output width in characters (default: 100)",
    )
    parser.add_argument(
        "-g", "--global-contrast",
        type=float,
        default=3.0,
        metavar="N",
        help="Global contrast enhancement exponent (default: 3.0)",
    )
    parser.add_argument(
        "-e", "--edge-contrast",
        type=float,
        default=3.0,
        metavar="N",
        help="Directional edge contrast exponent (default: 3.0)",
    )
    parser.add_argument(
        "-q", "--sample-quality",
        type=int,
        default=16,
        metavar="N",
        help="Samples per circle for quality (default: 16)",
    )
    parser.add_argument(
        "-i", "--invert",
        action="store_true",
        help="Invert luminance",
    )
    parser.add_argument(
        "--light-mode",
        action="store_true",
        help="Use dark text on light background instead of light on dark",
    )

    return parser.parse_args(args)


def main(args: list[str] | None = None) -> int:
    """Main entry point."""
    parsed = parse_args(args)

    config = RenderConfig(
        width=parsed.width,
        global_contrast=parsed.global_contrast,
        edge_contrast=parsed.edge_contrast,
        sample_quality=parsed.sample_quality,
        invert=parsed.invert,
    )

    try:
        ascii_art = render_file_to_ascii(parsed.image, config)
    except FileNotFoundError:
        print(f"Error: Image file not found: {parsed.image}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error processing image: {e}", file=sys.stderr)
        return 1

    if parsed.light_mode:
        fg_color = (40, 40, 40)
        bg_color = (255, 255, 255)
    else:
        fg_color = (180, 180, 180)
        bg_color = (0, 0, 0)

    try:
        render_to_image(ascii_art, parsed.output, fg_color=fg_color, bg_color=bg_color)
        print(f"Saved to {parsed.output}")
    except Exception as e:
        print(f"Error saving image: {e}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
