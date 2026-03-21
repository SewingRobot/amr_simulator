"""Main converter orchestrator for asset processing pipeline.

Receives conversion requests and dispatches to the appropriate
format-specific converter (e.g., URDF to glTF).
"""

import logging
from pathlib import Path
from enum import Enum

from .urdf_to_gltf import convert_urdf_to_gltf

logger = logging.getLogger(__name__)


class InputFormat(Enum):
    URDF = "urdf"


class OutputFormat(Enum):
    GLTF = "gltf"
    GLB = "glb"


class ConversionResult:
    """Result of an asset conversion operation."""

    def __init__(self, output_path: Path, success: bool, error: str | None = None):
        self.output_path = output_path
        self.success = success
        self.error = error


def convert(
    input_path: Path,
    output_dir: Path,
    input_format: InputFormat,
    output_format: OutputFormat = OutputFormat.GLB,
) -> ConversionResult:
    """Dispatch a conversion request to the appropriate converter.

    Args:
        input_path: Path to the source asset file.
        output_dir: Directory where converted output will be written.
        input_format: Format of the input file.
        output_format: Desired output format.

    Returns:
        ConversionResult with the output path and status.
    """
    logger.info(
        "Converting %s (%s) -> %s",
        input_path,
        input_format.value,
        output_format.value,
    )

    output_path = output_dir / f"{input_path.stem}.{output_format.value}"

    try:
        if input_format == InputFormat.URDF:
            convert_urdf_to_gltf(input_path, output_path)
            return ConversionResult(output_path=output_path, success=True)
        else:
            msg = f"Unsupported conversion: {input_format.value} -> {output_format.value}"
            logger.error(msg)
            return ConversionResult(output_path=output_path, success=False, error=msg)
    except Exception as e:
        logger.exception("Conversion failed")
        return ConversionResult(output_path=output_path, success=False, error=str(e))
