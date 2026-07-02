import argparse
import json
import math
from pathlib import Path
import sys

import bpy # pyright: ignore[reportMissingImports]


def get_camera_settings():
    scene = bpy.context.scene
    camera = scene.camera
    camera_data = camera.data

    name = camera.name
    location = [position for position in camera.location]
    rotation = [math.degrees(angle) for angle in camera.rotation_euler]

    focal_length = camera_data.lens
    sensor_width = camera_data.sensor_width

    dof = {
        "use_dof": camera_data.dof.use_dof,
        "focus_distance": camera_data.dof.focus_distance,
        "aperture_fstop": camera_data.dof.aperture_fstop,
    }

    render = {
        "resolution_x": scene.render.resolution_x,
        "resolution_y": scene.render.resolution_y,
    }

    return {
        "name": name,
        "location": location,
        "rotation": rotation,
        "focal_length": focal_length,
        "sensor_width": sensor_width,
        "dof": dof,
        "render": render,
    }

def export_camera(output_filepath='.'):
    camera_settings = get_camera_settings()
    path = Path(output_filepath) / 'camera.json'

    with open(path, "w", encoding="utf-8") as f:
        json.dump(camera_settings, f, indent=4)

    print(f"Exported camera: '{camera_settings["name"]}' to filepath: '{path}'.")


# run the following command with Blender on path:
# blender path_to_scene.blend --background --python scripts/export_blender_camera.py
if __name__ == '__main__':
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=str, default=".")
    args = parser.parse_args(argv)

    export_camera(args.output)