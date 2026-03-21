"""URDF to glTF/GLB conversion module."""

from pathlib import Path
import logging

logger = logging.getLogger(__name__)


def convert_urdf_to_gltf(urdf_path: Path, output_path: Path) -> Path:
    """Convert URDF file to glTF/GLB format.

    Args:
        urdf_path: Path to the input URDF file.
        output_path: Path where the glTF/GLB output will be written.

    Returns:
        Path to the generated output file.

    Raises:
        Exception: If conversion fails due to parsing or mesh errors.
    """
    logger.info("Converting %s to glTF", urdf_path)

    try:
        import urdfpy
        import trimesh

        robot = urdfpy.URDF.load(str(urdf_path))
        scene = trimesh.Scene()

        for link in robot.links:
            if link.visuals:
                for visual in link.visuals:
                    if visual.geometry.mesh:
                        mesh = trimesh.load(visual.geometry.mesh.filename)
                        scene.add_geometry(mesh, node_name=link.name)

        output_path.parent.mkdir(parents=True, exist_ok=True)
        scene.export(str(output_path))
        logger.info("Conversion complete: %s", output_path)
        return output_path

    except Exception as e:
        logger.error("Conversion failed: %s", e)
        raise


if __name__ == "__main__":
    import sys

    convert_urdf_to_gltf(Path(sys.argv[1]), Path(sys.argv[2]))
