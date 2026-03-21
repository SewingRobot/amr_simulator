"""Potree conversion module.

Converts processed point clouds into Potree octree tile format for
efficient streaming and visualization.

Currently a placeholder that saves the point cloud as a binary PLY file.
Full PotreeConverter integration will be added in a later phase.
"""

from pathlib import Path
import logging

import open3d as o3d

logger = logging.getLogger(__name__)


def convert(pcd: o3d.geometry.PointCloud, output_dir: Path) -> Path:
    """Convert a point cloud to Potree tile format.

    Currently saves the point cloud as a PLY file as a placeholder.
    Full PotreeConverter integration (octree generation) will be added later.

    Args:
        pcd: Preprocessed Open3D point cloud.
        output_dir: Directory to write output tiles.

    Returns:
        Path to the output tiles directory.
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Placeholder: save as PLY for now
    # TODO: Integrate PotreeConverter CLI for proper octree generation
    #   subprocess.run(["PotreeConverter", input_file, "-o", output_dir])
    output_file = output_dir / "pointcloud.ply"
    o3d.io.write_point_cloud(str(output_file), pcd)
    logger.info(
        "Saved point cloud to %s (%d points). "
        "Note: Full Potree octree conversion not yet implemented.",
        output_file,
        len(pcd.points),
    )

    return output_dir
