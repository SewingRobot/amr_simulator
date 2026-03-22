#!/usr/bin/env python3
"""Generate test fixtures for all data types."""
import json
import random
import math
import os
from pathlib import Path

SEED = 42
random.seed(SEED)

FIXTURES_DIR = Path(__file__).parent.parent / "fixtures"

def generate_robot_configs(count: int) -> list:
    robots = []
    for i in range(count):
        robots.append({
            "id": f"robot-{i+1:03d}",
            "name": f"TestBot-{i+1}",
            "model_id": "simple_diff_drive",
            "wheel_radius": 0.05,
            "wheel_separation": 0.3,
            "max_linear_speed": 1.5 + random.random(),
            "max_angular_speed": 2.0 + random.random(),
            "collision_radius": 0.25,
            "sensors": [{"type": "lidar_2d", "name": "front_lidar"}]
        })
    return robots

def generate_roadmap(node_count: int, world_w: float = 20, world_h: float = 15) -> dict:
    nodes = []
    for i in range(node_count):
        node_type = "waypoint"
        if i == 0: node_type = "waypoint"
        elif i == node_count - 1: node_type = "loading_dock"
        elif i == node_count - 2: node_type = "charging_station"

        nodes.append({
            "id": f"node-{i+1:03d}",
            "name": f"Node-{i+1}",
            "node_type": node_type,
            "x": random.uniform(1, world_w - 1),
            "y": random.uniform(1, world_h - 1),
            "z": 0.0,
        })

    edges = []
    # Connect sequential nodes
    for i in range(len(nodes) - 1):
        n1, n2 = nodes[i], nodes[i+1]
        dist = math.sqrt((n1["x"]-n2["x"])**2 + (n1["y"]-n2["y"])**2)
        edges.append({
            "id": f"edge-{i+1:03d}",
            "source": n1["id"],
            "target": n2["id"],
            "distance": round(dist, 2),
            "max_speed": 1.5,
            "direction": "bi",
        })
    # Add a few extra connections
    for _ in range(min(node_count // 3, 5)):
        i, j = random.sample(range(len(nodes)), 2)
        if any(e["source"]==nodes[i]["id"] and e["target"]==nodes[j]["id"] for e in edges):
            continue
        n1, n2 = nodes[i], nodes[j]
        dist = math.sqrt((n1["x"]-n2["x"])**2 + (n1["y"]-n2["y"])**2)
        edges.append({
            "id": f"edge-extra-{i}-{j}",
            "source": n1["id"],
            "target": n2["id"],
            "distance": round(dist, 2),
            "max_speed": 1.0,
            "direction": "bi",
        })

    return {"nodes": nodes, "edges": edges}

def generate_telemetry_sequence(robot_id: str, steps: int = 100, dt: float = 0.1) -> list:
    """Generate a sequence of telemetry messages for a robot moving in a line."""
    msgs = []
    x, y, theta = 5.0, 5.0, 0.0
    v, w = 0.5, 0.1
    battery = 100.0
    for i in range(steps):
        x += v * math.cos(theta) * dt
        y += v * math.sin(theta) * dt
        theta += w * dt
        battery -= 0.01
        msgs.append({
            "robot_id": robot_id,
            "timestamp_ms": 1700000000000 + i * int(dt * 1000),
            "pose": {"x": round(x, 4), "y": round(y, 4), "theta": round(theta, 4)},
            "velocity": {"linear": v, "angular": w},
            "battery_percent": round(battery, 2),
            "status": "busy",
            "errors": [],
        })
    return msgs

def generate_world(name: str, w: float = 20, h: float = 15, obstacles: int = 3) -> dict:
    world = {
        "world": {
            "name": name,
            "size": [w, h],
            "walls": [
                {"from": [0, 0], "to": [w, 0]},
                {"from": [w, 0], "to": [w, h]},
                {"from": [w, h], "to": [0, h]},
                {"from": [0, h], "to": [0, 0]},
            ],
            "obstacles": []
        }
    }
    for i in range(obstacles):
        world["world"]["obstacles"].append({
            "type": "box",
            "position": [random.uniform(3, w-3), random.uniform(3, h-3)],
            "size": [random.uniform(1, 3), 0.2],
        })
    return world

def generate_missions(count: int, node_ids: list) -> list:
    missions = []
    for i in range(count):
        start, end = random.sample(node_ids, 2)
        missions.append({
            "id": f"mission-{i+1:03d}",
            "start_node_id": start,
            "end_node_id": end,
            "priority": random.randint(0, 10),
            "status": "created",
        })
    return missions

def save(data, path: Path):
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"  Generated: {path}")

def main():
    print("Generating test fixtures (seed={})...".format(SEED))

    # XS dataset
    xs = FIXTURES_DIR / "xs"
    robots_xs = generate_robot_configs(1)
    roadmap_xs = generate_roadmap(3)
    save(robots_xs, xs / "robots.json")
    save(roadmap_xs, xs / "roadmap.json")
    save(generate_world("xs_room", 10, 8, 1), xs / "world.json")
    save(generate_telemetry_sequence("robot-001", 50), xs / "telemetry.json")
    save(generate_missions(1, [n["id"] for n in roadmap_xs["nodes"]]), xs / "missions.json")

    # S dataset
    s = FIXTURES_DIR / "s"
    robots_s = generate_robot_configs(3)
    roadmap_s = generate_roadmap(10)
    save(robots_s, s / "robots.json")
    save(roadmap_s, s / "roadmap.json")
    save(generate_world("s_warehouse", 20, 15, 3), s / "world.json")
    save(generate_telemetry_sequence("robot-001", 100), s / "telemetry_r1.json")
    save(generate_telemetry_sequence("robot-002", 100), s / "telemetry_r2.json")
    save(generate_telemetry_sequence("robot-003", 100), s / "telemetry_r3.json")
    save(generate_missions(5, [n["id"] for n in roadmap_s["nodes"]]), s / "missions.json")

    print("Done!")

if __name__ == "__main__":
    main()
