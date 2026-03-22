#!/usr/bin/env python3
"""Mock WebSocket server that sends fake telemetry at 10Hz. For frontend dev without backend."""
import asyncio
import json
import math
import time

try:
    import websockets
except ImportError:
    print("pip install websockets")
    exit(1)

robots = [
    {"id": "mock-robot-1", "x": 5, "y": 5, "theta": 0, "v": 0.8, "w": 0.3},
    {"id": "mock-robot-2", "x": 10, "y": 3, "theta": 1.5, "v": 0.5, "w": -0.2},
]

async def telemetry_stream(websocket):
    print(f"Client connected")
    # Wait for subscribe message
    try:
        msg = await asyncio.wait_for(websocket.recv(), timeout=5)
        print(f"Received: {msg}")
    except:
        pass

    t = 0
    while True:
        for r in robots:
            r["x"] += r["v"] * math.cos(r["theta"]) * 0.1
            r["y"] += r["v"] * math.sin(r["theta"]) * 0.1
            r["theta"] += r["w"] * 0.1
            # Clamp to bounds
            r["x"] = max(0.5, min(19.5, r["x"]))
            r["y"] = max(0.5, min(14.5, r["y"]))

            half_theta = r["theta"] / 2
            envelope = {
                "type": "telemetry",
                "topic": f"telemetry:{r['id']}",
                "timestamp": time.time() * 1000,
                "payload": {
                    "robot_id": r["id"],
                    "timestamp_ms": time.time() * 1000,
                    "pose": {
                        "position": {"x": r["x"], "y": r["y"], "z": 0},
                        "orientation": {"x": 0, "y": 0, "z": math.sin(half_theta), "w": math.cos(half_theta)},
                    },
                    "velocity": {
                        "linear": {"x": r["v"], "y": 0, "z": 0},
                        "angular": {"x": 0, "y": 0, "z": r["w"]},
                    },
                    "battery_percent": 95 - t * 0.01,
                    "status": "busy",
                    "errors": [],
                },
            }
            await websocket.send(json.dumps(envelope))

        t += 0.1
        await asyncio.sleep(0.1)

async def main():
    print("Mock WS server on ws://localhost:8081/api/ws")
    async with websockets.serve(telemetry_stream, "0.0.0.0", 8081):
        await asyncio.Future()

if __name__ == "__main__":
    asyncio.run(main())
