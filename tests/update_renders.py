import subprocess

import pytest

from conftest import RENDER_SCENES


@pytest.mark.update_renders
@pytest.mark.parametrize("name, samples, resolution", RENDER_SCENES)
def test_create_new_renders(name, samples, resolution, image_dir, tmp_path, rust_binary, read_exr):
    """Create new renders to test against.
    
    Run by calling pytest -m update_renders.
    This is needed for when the renderer's output is changed due to
    a change in code that affects visuals.
    """
    print(f"Creating new render for {name} scene.")
    
    reference_path = image_dir / f"{name}.exr"
    
    subprocess.run(
        [
            str(rust_binary),
            '--output', str(reference_path),
            '--samples', str(samples),
            '--width', str(resolution[0]),
            '--height', str(resolution[1]),
            '--scene', str(name),
        ],
        check=True)