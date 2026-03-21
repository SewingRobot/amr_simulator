"""Main processing orchestrator for point cloud data.

Pipeline stages:
  1. Ingest: Load point cloud from LAS/PLY/PCD file
  2. Preprocess: Downsample, remove outliers, estimate normals
  3. Potree convert: Generate octree tiles for streaming
"""

from pathlib import Path
from dataclasses import dataclass
import logging

logger = logging.getLogger(__name__)


@dataclass
class PipelineConfig:
    """Configuration for the point cloud processing pipeline."""

    voxel_size: float = 0.02
    outlier_neighbors: int = 20
    outlier_std_ratio: float = 2.0


def process_pointcloud(
    input_path: Path,
    output_dir: Path,
    config: PipelineConfig = PipelineConfig(),
) -> dict:
    """Full pipeline: ingest -> preprocess -> potree convert.

    Args:
        input_path: Path to the input point cloud file (.ply, .pcd, .las, .laz).
        output_dir: Directory to write processed output tiles.
        config: Pipeline configuration parameters.

    Returns:
        Dictionary with processing results including point_count, tiles_path, and bounds.
    """
    from . import ingest, preprocess, potree_convert

    logger.info("Starting pipeline for %s", input_path)

    # 1. Ingest
    pcd = ingest.load_pointcloud(input_path)
    logger.info("Loaded %d points", len(pcd.points))

    # 2. Preprocess
    pcd = preprocess.downsample(pcd, config.voxel_size)
    pcd = preprocess.remove_outliers(
        pcd, config.outlier_neighbors, config.outlier_std_ratio
    )
    pcd = preprocess.estimate_normals(pcd)
    logger.info("After preprocessing: %d points", len(pcd.points))

    # 3. Potree convert
    tiles_dir = potree_convert.convert(pcd, output_dir)

    return {
        "point_count": len(pcd.points),
        "tiles_path": str(tiles_dir),
        "bounds": pcd.get_axis_aligned_bounding_box(),
    }
