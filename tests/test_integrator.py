"""
Renders new images and compares them to the renders
in the images directory. Must be in the project's
root directory for the tests to work.
"""

import subprocess

import pytest
from skimage.metrics import structural_similarity as ssim

from conftest import RENDER_SCENES

 
@pytest.mark.parametrize("name, samples, resolution", RENDER_SCENES)
def test_render_output_matches_reference(name, samples, resolution, image_dir, tmp_path, rust_binary, read_exr):
    print(f"Testing: {name}")

    reference_path = image_dir / f"{name}.exr"
    output_path = tmp_path / f"{name}.exr"
    
    subprocess.run(
        [
            str(rust_binary),
            '--output', str(output_path),
            '--samples', str(samples),
            '--width', str(resolution[0]),
            '--height', str(resolution[1]),
            '--scene', str(name),
        ],
        check=True)
    
    new_img = read_exr(output_path)
    ref_img = read_exr(reference_path)

    assert new_img.shape == ref_img.shape
    
    drange = ref_img.max() - ref_img.min()
    score = ssim(
        ref_img, 
        new_img, 
        channel_axis=-1, 
        data_range=drange, 
    )

    print(f"SSIM Score: {score:.4f}")

    assert score > 0.9997