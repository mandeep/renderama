import os
os.environ.setdefault("OPENCV_IO_ENABLE_OPENEXR", "1")

import argparse
import sys

import cv2
import numpy as np


def linear_to_srgb(x: np.ndarray) -> np.ndarray:
    x = np.clip(x, 0, None)
    return np.where(
        x <= 0.0031308,
        x * 12.92,
        1.055 * np.power(x, 1.0 / 2.4) - 0.055,
    )


def convert(input_path: str) -> None:
    img = cv2.imread(input_path, cv2.IMREAD_UNCHANGED)
    if img is None:
        sys.exit(f"Error: could not read '{input_path}' as an EXR. "
                  f"Check the path and that the file is a valid EXR.")

    if img.dtype != np.float32:
        img = img.astype(np.float32)

    srgb = linear_to_srgb(img)
    srgb_8bit = np.clip(srgb * 255.0 + 0.5, 0, 255).astype(np.uint8)

    output_path = os.path.splitext(input_path)[0] + ".png"
    ok = cv2.imwrite(output_path, srgb_8bit)
    if not ok:
        sys.exit(f"Error: failed to write '{output_path}'.")

    print(f"Saved png file to {output_path}.")


def main():
    parser = argparse.ArgumentParser(
        description="Convert a linear EXR to an 8-bit sRGB PNG (no tone mapping)."
    )
    parser.add_argument("input", help="Path to input .exr file")
    args = parser.parse_args()

    convert(args.input)


if __name__ == "__main__":
    main()