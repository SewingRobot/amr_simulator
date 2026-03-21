"""Point cloud ingestion module.

Supports loading point cloud data from various formats:
  - PLY (Polygon File Format)
  - PCD (Point Cloud Data)
  - LAS/LAZ (LASer file format)
"""

from pathlib import Path
import logging

import numpy as np
import open3d as o3d

logger = logging.getLogger(__name__)


def load_pointcloud(path: Path) -> o3d.geometry.PointCloud:
    """Load a point cloud from file.

    Supported formats: .ply, .pcd, .las, .laz

    Args:
        path: Path to the point cloud file.

    Returns:
        An Open3D PointCloud object.

    Raises:
        ValueError: If the file format is not supported.
        FileNotFoundError: If the file does not exist.
    """
    path = Path(path)

    if not path.exists():
        raise FileNotFoundError(f"Point cloud file not found: {path}")

    suffix = path.suffix.lower()
    logger.info("Loading point cloud from %s (format: %s)", path, suffix)

    if suffix in (".ply", ".pcd"):
        pcd = o3d.io.read_point_cloud(str(path))
    elif suffix in (".las", ".laz"):
        pcd = _load_las(path)
    else:
        raise ValueError(
            f"Unsupported point cloud format: {suffix}. "
            f"Supported formats: .ply, .pcd, .las, .laz"
        )

    logger.info("Loaded %d points from %s", len(pcd.points), path.name)
    return pcd


def _load_las(path: Path) -> o3d.geometry.PointCloud:
    """Load a LAS/LAZ file using laspy and convert to Open3D PointCloud.

    Args:
        path: Path to the LAS/LAZ file.

    Returns:
        An Open3D PointCloud object.
    """
    import laspy

    las = laspy.read(str(path))

    points = np.vstack((las.x, las.y, las.z)).T

    pcd = o3d.geometry.PointCloud()
    pcd.points = o3d.utility.Vector3dVector(points)

    # Load colors if available
    if hasattr(las, "red") and hasattr(las, "green") and hasattr(las, "blue"):
        colors = (
            np.vstack((las.red, las.green, las.blue)).T.astype(np.float64) / 65535.0
        )
        pcd.colors = o3d.utility.Vector3dVector(colors)

    return pcd
