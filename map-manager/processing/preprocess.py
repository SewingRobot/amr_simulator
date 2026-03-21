"""Point cloud preprocessing module.

Provides functions for:
  - Voxel downsampling
  - Statistical outlier removal
  - Normal estimation
"""

import logging

import open3d as o3d

logger = logging.getLogger(__name__)


def downsample(pcd: o3d.geometry.PointCloud, voxel_size: float) -> o3d.geometry.PointCloud:
    """Downsample a point cloud using voxel grid filtering.

    Args:
        pcd: Input point cloud.
        voxel_size: Voxel size in meters (e.g., 0.02 for 2cm resolution).

    Returns:
        Downsampled point cloud.
    """
    original_count = len(pcd.points)
    pcd_down = pcd.voxel_down_sample(voxel_size=voxel_size)
    logger.info(
        "Downsampled from %d to %d points (voxel_size=%.3f)",
        original_count,
        len(pcd_down.points),
        voxel_size,
    )
    return pcd_down


def remove_outliers(
    pcd: o3d.geometry.PointCloud,
    nb_neighbors: int,
    std_ratio: float,
) -> o3d.geometry.PointCloud:
    """Remove statistical outliers from a point cloud.

    Args:
        pcd: Input point cloud.
        nb_neighbors: Number of neighbors to consider for each point.
        std_ratio: Standard deviation ratio threshold. Points with a distance
            larger than std_ratio * std_dev are considered outliers.

    Returns:
        Point cloud with outliers removed.
    """
    original_count = len(pcd.points)
    pcd_clean, _indices = pcd.remove_statistical_outlier(
        nb_neighbors=nb_neighbors,
        std_ratio=std_ratio,
    )
    removed = original_count - len(pcd_clean.points)
    logger.info(
        "Removed %d outliers (%d remaining, nn=%d, std_ratio=%.1f)",
        removed,
        len(pcd_clean.points),
        nb_neighbors,
        std_ratio,
    )
    return pcd_clean


def estimate_normals(
    pcd: o3d.geometry.PointCloud,
    search_radius: float = 0.1,
    max_nn: int = 30,
) -> o3d.geometry.PointCloud:
    """Estimate point normals using local neighborhood.

    Args:
        pcd: Input point cloud.
        search_radius: Search radius for normal estimation (meters).
        max_nn: Maximum number of neighbors to use.

    Returns:
        Point cloud with estimated normals.
    """
    pcd.estimate_normals(
        search_param=o3d.geometry.KDTreeSearchParamHybrid(
            radius=search_radius,
            max_nn=max_nn,
        )
    )
    logger.info("Estimated normals for %d points", len(pcd.points))
    return pcd
