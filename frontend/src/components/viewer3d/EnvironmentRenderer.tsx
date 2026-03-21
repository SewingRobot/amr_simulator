import { useViewerStore } from '../../stores/viewerStore'

// World dimensions matching sim-engine basic_warehouse.json
// Sim coords: x: 0→20, y: 0→15 (ENU)
// Three.js:   x: 0→20, z: 0→-15 (enuToThreeJS: z = -y)
const WORLD_W = 20
const WORLD_H = 15
const WALL_THICKNESS = 0.2
const WALL_HEIGHT = 2

// Center offsets for Three.js positioning
const CX = WORLD_W / 2   // 10
const CZ = -WORLD_H / 2  // -7.5

export function EnvironmentRenderer() {
  const viewMode = useViewerStore((state) => state.viewMode)

  return (
    <group>
      {/* Grid centered on the world */}
      <gridHelper args={[WORLD_W, WORLD_W, '#444444', '#222222']} position={[CX, 0, CZ]} />
      <axesHelper args={[3]} />

      {viewMode === '3d' ? (
        <group>
          {/* Bottom wall (y=0 → z=0) */}
          <mesh position={[CX, WALL_HEIGHT / 2, 0]}>
            <boxGeometry args={[WORLD_W, WALL_HEIGHT, WALL_THICKNESS]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
          {/* Top wall (y=15 → z=-15) */}
          <mesh position={[CX, WALL_HEIGHT / 2, -WORLD_H]}>
            <boxGeometry args={[WORLD_W, WALL_HEIGHT, WALL_THICKNESS]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
          {/* Left wall (x=0) */}
          <mesh position={[0, WALL_HEIGHT / 2, CZ]}>
            <boxGeometry args={[WALL_THICKNESS, WALL_HEIGHT, WORLD_H]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
          {/* Right wall (x=20) */}
          <mesh position={[WORLD_W, WALL_HEIGHT / 2, CZ]}>
            <boxGeometry args={[WALL_THICKNESS, WALL_HEIGHT, WORLD_H]} />
            <meshStandardMaterial color="#666666" />
          </mesh>

          {/* Shelf obstacles from warehouse (3 shelves at y=7 → z=-7) */}
          {[5, 10, 15].map((ox) => (
            <mesh key={ox} position={[ox, 1.25, -7]}>
              <boxGeometry args={[4, 2.5, 0.2]} />
              <meshStandardMaterial color="#aa6633" />
            </mesh>
          ))}
        </group>
      ) : (
        <group>
          {/* 2D flat walls */}
          <mesh position={[CX, 0.01, 0]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[WORLD_W, 0.3]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
          <mesh position={[CX, 0.01, -WORLD_H]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[WORLD_W, 0.3]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
          <mesh position={[0, 0.01, CZ]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[0.3, WORLD_H]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
          <mesh position={[WORLD_W, 0.01, CZ]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[0.3, WORLD_H]} />
            <meshStandardMaterial color="#888888" />
          </mesh>

          {/* 2D shelf obstacles */}
          {[5, 10, 15].map((ox) => (
            <mesh key={ox} position={[ox, 0.02, -7]} rotation={[-Math.PI / 2, 0, 0]}>
              <planeGeometry args={[4, 0.3]} />
              <meshStandardMaterial color="#aa6633" />
            </mesh>
          ))}
        </group>
      )}
    </group>
  )
}
