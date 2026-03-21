import { useViewerStore } from '../../stores/viewerStore'

export function EnvironmentRenderer() {
  const viewMode = useViewerStore((state) => state.viewMode)

  return (
    <group>
      {/* 20x20 grid with 1m spacing */}
      <gridHelper args={[20, 20, '#444444', '#222222']} />
      {/* Axes: X=red, Y=green, Z=blue */}
      <axesHelper args={[5]} />

      {/* Example walls - render differently based on view mode */}
      {viewMode === '3d' ? (
        <group>
          {/* 3D walls with height */}
          <mesh position={[-10, 1, 0]}>
            <boxGeometry args={[0.2, 2, 20]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
          <mesh position={[10, 1, 0]}>
            <boxGeometry args={[0.2, 2, 20]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
          <mesh position={[0, 1, -10]}>
            <boxGeometry args={[20, 2, 0.2]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
          <mesh position={[0, 1, 10]}>
            <boxGeometry args={[20, 2, 0.2]} />
            <meshStandardMaterial color="#666666" />
          </mesh>
        </group>
      ) : (
        <group>
          {/* 2D walls as flat colored rectangles */}
          <mesh position={[-10, 0.01, 0]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[0.3, 20]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
          <mesh position={[10, 0.01, 0]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[0.3, 20]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
          <mesh position={[0, 0.01, -10]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[20, 0.3]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
          <mesh position={[0, 0.01, 10]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[20, 0.3]} />
            <meshStandardMaterial color="#888888" />
          </mesh>
        </group>
      )}
    </group>
  )
}
