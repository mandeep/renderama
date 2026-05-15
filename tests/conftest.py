import os
from pathlib import Path
import subprocess

os.environ["OPENCV_IO_ENABLE_OPENEXR"] = "1"
import cv2
import numpy as np
import pytest
from skimage.metrics import structural_similarity as ssim


RENDER_SCENES = [
    # scene name and number of samples to render with
    ("cornell_box_dragon", 128, (512, 512)),
    ("cornell_box_boxes", 64, (512, 512)),
    ("cornell_box_bunny", 128, (512, 512)),
    ("cornell_box_objects", 128, (512, 512)),
    ("energy_conservation", 64, (2048, 512)),
    ("three_spheres", 64, (1024, 512)),
    ("veach_mis", 64, (1920, 1080)),
    ("white_furnace", 64, (2048, 512)),
]

@pytest.fixture(scope="session", autouse=True)
def compile_binary():
    """Compiles the Rust binary with test_mode once per session."""
    print("\n[Cargo] Compiling as `cargo build --features tests`...")
    subprocess.run(["cargo", "build", "--features", "tests"], check=True)


@pytest.fixture
def rust_binary():
    """Returns the path to the compiled executable."""
    ext = ".exe" if os.name == "nt" else ""
    return Path(__file__).parent.parent / "target" / "debug" / f"renderama{ext}"


@pytest.fixture
def read_exr():
    """Loads the reference exr file."""
    def read(path):
        img = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
        if img is None:
            raise FileNotFoundError(f"Could not load image at {path}")

        img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
        return img.astype(np.float32)
    return read


@pytest.fixture
def image_dir():
    """Returns the absolute path to the tests/images directory."""
    return Path(__file__).parent / "images"
