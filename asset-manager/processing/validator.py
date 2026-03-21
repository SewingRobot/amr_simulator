"""Asset validation module.

Validates uploaded asset files for integrity, format correctness,
and adherence to project constraints.
"""

import logging
from dataclasses import dataclass, field
from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class ValidationResult:
    """Result of an asset validation check."""

    valid: bool
    warnings: list[str] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)


def validate_asset(file_path: Path) -> ValidationResult:
    """Validate an asset file.

    Performs basic checks including:
    - File existence
    - Non-zero file size
    - Extension recognition

    Args:
        file_path: Path to the asset file to validate.

    Returns:
        ValidationResult with warnings and errors lists.
    """
    warnings: list[str] = []
    errors: list[str] = []

    # Check file exists
    if not file_path.exists():
        errors.append(f"File does not exist: {file_path}")
        return ValidationResult(valid=False, warnings=warnings, errors=errors)

    # Check file is not empty
    size = file_path.stat().st_size
    if size == 0:
        errors.append("File is empty (0 bytes)")
        return ValidationResult(valid=False, warnings=warnings, errors=errors)

    # Warn on large files (> 100 MB)
    if size > 100 * 1024 * 1024:
        warnings.append(f"File is large ({size / (1024*1024):.1f} MB)")

    # Check known extensions
    known_extensions = {
        ".urdf", ".xacro", ".sdf",
        ".stl", ".dae", ".obj",
        ".gltf", ".glb",
        ".json", ".yaml", ".yml",
    }
    suffix = file_path.suffix.lower()
    if suffix not in known_extensions:
        warnings.append(f"Unrecognized file extension: {suffix}")

    # Format-specific validation
    if suffix == ".urdf":
        _validate_urdf(file_path, warnings, errors)

    valid = len(errors) == 0
    logger.info(
        "Validation %s for %s (%d warnings, %d errors)",
        "passed" if valid else "failed",
        file_path,
        len(warnings),
        len(errors),
    )
    return ValidationResult(valid=valid, warnings=warnings, errors=errors)


def _validate_urdf(file_path: Path, warnings: list[str], errors: list[str]) -> None:
    """Basic URDF-specific validation checks."""
    try:
        content = file_path.read_text(encoding="utf-8")
        if "<robot" not in content:
            errors.append("URDF file does not contain a <robot> element")
        if 'name="' not in content and "name='" not in content:
            warnings.append("URDF <robot> element may be missing a name attribute")
    except UnicodeDecodeError:
        errors.append("URDF file is not valid UTF-8 text")
