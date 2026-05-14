"""
Renders new images and compares them to the renders
in the images directory. Must be in the project's
root directory for the tests to work.
"""

import subprocess
import os
from pathlib import Path

os.environ["OPENCV_IO_ENABLE_OPENEXR"] = "1"
import cv2

import pytest
import imageio.v3 as iio
from skimage.metrics import structural_similarity as ssim
import numpy as np


RENDER_SCENES = [
    # scene name and number of samples to render with
    ("cornell_box_dragon", 256),
    ("cornell_box_boxes", 256),
    ("cornell_box_bunny", 256),
    ("cornell_box_objects", 256),
    ("energy_conservation", 32),
    ("three_spheres", 256),
    ("veach_mis", 32),
    ("white_furnace", 32),
]


@pytest.fixture(scope="session", autouse=True)
def compile_binary():
    """Compiles the Rust binary with test_mode once per session."""
    print("\n[Cargo] Compiling in test_mode...")
    subprocess.run(["cargo", "build", "--features", "tests"], check=True)


@pytest.fixture
def rust_binary():
    """Returns the path to the compiled executable."""
    ext = ".exe" if os.name == "nt" else ""
    return Path(__file__).parent.parent / "target" / "debug" / f"renderama{ext}"


@pytest.fixture
def read_exr():
    """Loads the reference exr file."""
    def _read(path):
        img = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
        if img is None:
            raise FileNotFoundError(f"Could not load image at {path}")

        img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
        return img.astype(np.float32)
    return _read


@pytest.fixture
def image_dir():
    """Returns the absolute path to the tests/images directory."""
    return Path(__file__).parent / "images"

 
@pytest.mark.parametrize("name, samples", RENDER_SCENES)
def test_render_output_matches_reference(name, samples, image_dir, tmp_path, rust_binary, read_exr):
    print(f"Testing: {name}")

    reference_path = image_dir / f"{name}.exr"
    output_path = tmp_path / f"{name}.exr"
    
    subprocess.run(
        [
            str(rust_binary),
            '--output', str(output_path),
            '--samples', str(samples),
            '--resolution', '256', '256',
            '--scene', str(name),
        ],
        check=True)
    
    new_img = read_exr(output_path)
    ref_img = read_exr(reference_path)

    assert new_img.shape == ref_img.shape
    
    drange = ref_img.max() - ref_img.min()
    score, diff = ssim(
        ref_img, 
        new_img, 
        channel_axis=-1, 
        data_range=drange, 
        full=True
    )

    print(f"SSIM Score: {score:.4f}")

    assert score > 0.9998