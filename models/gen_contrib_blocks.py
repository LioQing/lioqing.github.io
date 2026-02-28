from __future__ import annotations

import csv
from pathlib import Path

import bmesh  # type: ignore[import-not-found]
import bpy  # type: ignore[import-not-found]


CSV_PATH = ""
COLLECTION_NAME = "ContribBlocks"
MATERIAL_NAME = "ContribMat"
BLOCK_SIZE_X = 0.8
BLOCK_SIZE_Y = 0.8
ROUND_TOP_ENABLED = True
ROUND_TOP_OFFSET = 0.2
ROUND_TOP_SEGMENTS = 5


def _resolve_csv_path() -> Path:
    if CSV_PATH:
        return Path(CSV_PATH).expanduser().resolve()

    blend_path = Path(bpy.data.filepath) if bpy.data.filepath else None
    if blend_path and blend_path.exists():
        return (blend_path.parent / ".." / "data" / "contrib.csv").resolve()

    raise RuntimeError(
        "Unable to resolve contrib.csv path. Set CSV_PATH at top of script."
    )


def _parse_rows(csv_path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []

    with csv_path.open("r", encoding="utf-8", newline="") as file:
        reader = csv.DictReader(file)
        expected = {"date", "github", "gitlab", "x", "y"}
        fieldnames = {name.strip() for name in (reader.fieldnames or [])}
        if fieldnames != expected:
            raise RuntimeError(
                "Invalid CSV header. Expected columns: date,github,gitlab,x,y"
            )

        for line_number, row in enumerate(reader, start=2):
            try:
                date_value = (row.get("date") or "").strip()
                github_count = int((row.get("github") or "0").strip())
                gitlab_count = int((row.get("gitlab") or "0").strip())
                x_coord = float((row.get("x") or "0").strip())
                y_coord = float((row.get("y") or "0").strip())
            except ValueError as exc:
                raise RuntimeError(
                    f"Invalid value at CSV line {line_number}: {exc}"
                ) from exc

            if not date_value:
                raise RuntimeError(f"Missing date at CSV line {line_number}")

            total = github_count + gitlab_count
            if total <= 0:
                continue

            height = float(total)
            git_lab_hub_factor = github_count / total

            rows.append(
                {
                    "date": date_value,
                    "x": x_coord,
                    "y": y_coord,
                    "height": height,
                    "total": total,
                    "git_lab_hub_factor": git_lab_hub_factor,
                }
            )

    return rows


def _get_or_create_collection(name: str) -> bpy.types.Collection:
    collection = bpy.data.collections.get(name)
    if collection is None:
        collection = bpy.data.collections.new(name)
        bpy.context.scene.collection.children.link(collection)
    return collection


def _clear_collection(collection: bpy.types.Collection) -> None:
    for obj in list(collection.objects):
        bpy.data.objects.remove(obj, do_unlink=True)


def _create_block(
    collection: bpy.types.Collection,
    material: bpy.types.Material,
    date_value: str,
    x_coord: float,
    y_coord: float,
    height: float,
    total: int,
    git_lab_hub_factor: float,
) -> None:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x_coord, y_coord, height / 2.0))
    obj = bpy.context.active_object
    if obj is None:
        raise RuntimeError("Failed to create cube object")

    mesh = obj.data
    if not isinstance(mesh, bpy.types.Mesh):
        raise RuntimeError("Generated object is not a mesh")

    bm = bmesh.new()
    bm.from_mesh(mesh)
    bm.verts.ensure_lookup_table()
    bm.edges.ensure_lookup_table()

    bmesh.ops.scale(
        bm,
        vec=(BLOCK_SIZE_X, BLOCK_SIZE_Y, height),
        verts=bm.verts,
    )

    if ROUND_TOP_ENABLED:
        max_z = max(vertex.co.z for vertex in bm.verts)
        z_epsilon = 1e-6

        top_edges = [
            edge
            for edge in bm.edges
            if all(abs(vertex.co.z - max_z) <= z_epsilon for vertex in edge.verts)
        ]
        if top_edges:
            fixed_offset = min(
                ROUND_TOP_OFFSET, BLOCK_SIZE_X * 0.49, BLOCK_SIZE_Y * 0.49
            )
            bmesh.ops.bevel(
                bm,
                geom=top_edges,
                offset=fixed_offset,
                offset_type="OFFSET",
                segments=ROUND_TOP_SEGMENTS,
                profile=0.5,
                affect="EDGES",
            )

    bm.to_mesh(mesh)
    bm.free()
    mesh.update()

    obj.name = f"Contrib_{date_value}_{total}"
    obj["GitLabHubFactor"] = max(0.0, min(1.0, float(git_lab_hub_factor)))

    if mesh.materials:
        mesh.materials[0] = material
    else:
        mesh.materials.append(material)

    for parent_collection in list(obj.users_collection):
        parent_collection.objects.unlink(obj)

    if collection.objects.get(obj.name) is None:
        collection.objects.link(obj)


def main() -> None:
    csv_path = _resolve_csv_path()
    if not csv_path.exists():
        raise RuntimeError(f"CSV file not found: {csv_path}")

    rows = _parse_rows(csv_path)
    collection = _get_or_create_collection(COLLECTION_NAME)
    _clear_collection(collection)
    material = bpy.data.materials.get(MATERIAL_NAME)
    if material is None:
        raise RuntimeError(
            f'Material "{MATERIAL_NAME}" not found in current Blender file'
        )

    for row in rows:
        _create_block(
            collection=collection,
            material=material,
            date_value=str(row["date"]),
            x_coord=float(row["x"]),
            y_coord=float(row["y"]),
            height=float(row["height"]),
            total=int(row["total"]),
            git_lab_hub_factor=float(row["git_lab_hub_factor"]),
        )

    print(f"Generated {len(rows)} blocks from {csv_path}")


if __name__ == "__main__":
    main()
