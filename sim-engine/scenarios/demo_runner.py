#!/usr/bin/env python3
"""
Toy simulation demo: spawns robots and makes them drive around a warehouse.
Connects to the sim-engine's CommandServer (TCP port 50052).

Usage:
    # First start sim-engine:
    #   ./build/Release/sim-engine --world scenarios/basic_warehouse.json
    # Then run this script:
    #   python3 scenarios/demo_runner.py

    # Or all-in-one:
    #   ./build/Release/sim-engine --world scenarios/basic_warehouse.json &
    #   python3 scenarios/demo_runner.py
"""

import socket
import json
import time
import math
import sys

COMMAND_PORT = 50052
HOST = "127.0.0.1"


def send_command(sock: socket.socket, cmd: dict) -> dict:
    """Send a JSON command and receive response."""
    msg = json.dumps(cmd) + "\n"
    sock.sendall(msg.encode())
    # Read response line
    data = b""
    while b"\n" not in data:
        chunk = sock.recv(4096)
        if not chunk:
            break
        data += chunk
    if data:
        return json.loads(data.decode().strip())
    return {}


def main():
    print("🤖 AMR Toy Simulation Demo")
    print("=" * 40)

    # Connect to command server
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((HOST, COMMAND_PORT))
        print(f"✅ Connected to sim-engine at {HOST}:{COMMAND_PORT}")
    except ConnectionRefusedError:
        print(f"❌ Cannot connect to sim-engine at {HOST}:{COMMAND_PORT}")
        print("   Start sim-engine first:")
        print("   ./build/Release/sim-engine --world scenarios/basic_warehouse.json")
        sys.exit(1)

    # Spawn 3 robots at different positions
    robots = [
        {"id": "robot-1", "x": 2.0, "y": 2.0, "color": "blue"},
        {"id": "robot-2", "x": 10.0, "y": 2.0, "color": "green"},
        {"id": "robot-3", "x": 5.0, "y": 12.0, "color": "red"},
    ]

    for robot in robots:
        resp = send_command(sock, {
            "type": "spawn_robot",
            "robot_id": robot["id"],
            "x": robot["x"],
            "y": robot["y"],
            "theta": 0.0,
            "config": {
                "wheel_radius": 0.05,
                "wheel_separation": 0.3,
                "max_linear_speed": 1.5,
                "max_angular_speed": 2.0,
                "collision_radius": 0.25,
            }
        })
        print(f"  Spawned {robot['id']} at ({robot['x']}, {robot['y']}): {resp.get('status', 'ok')}")

    # Start simulation
    resp = send_command(sock, {"type": "start_sim"})
    print(f"\n▶️  Simulation started: {resp.get('status', 'ok')}")
    print(f"   Open http://localhost:3000 to see robots moving!")
    print(f"   (Backend must be running on port 8080)")
    print()

    # Drive robots in patterns
    try:
        t = 0
        dt = 0.5  # command update interval
        print("🔄 Sending velocity commands (Ctrl+C to stop)...")

        while True:
            # Robot 1: drive in a circle
            send_command(sock, {
                "type": "send_command",
                "robot_id": "robot-1",
                "linear": 0.8,
                "angular": 0.5,
            })

            # Robot 2: drive forward and back (oscillate)
            phase = math.sin(t * 0.5)
            send_command(sock, {
                "type": "send_command",
                "robot_id": "robot-2",
                "linear": phase * 1.0,
                "angular": 0.0,
            })

            # Robot 3: drive in a figure-8
            send_command(sock, {
                "type": "send_command",
                "robot_id": "robot-3",
                "linear": 0.6,
                "angular": math.sin(t * 0.3) * 1.5,
            })

            time.sleep(dt)
            t += dt

            # Print status every 5 seconds
            if int(t) % 5 == 0 and abs(t - int(t)) < dt:
                resp = send_command(sock, {"type": "get_state"})
                print(f"  t={t:.0f}s | robots={resp.get('robot_count', '?')} | "
                      f"sim_time={resp.get('sim_time_ms', 0)/1000:.1f}s")

    except KeyboardInterrupt:
        print("\n\n⏹️  Stopping simulation...")
        send_command(sock, {"type": "stop_sim"})
        print("   Simulation stopped.")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
