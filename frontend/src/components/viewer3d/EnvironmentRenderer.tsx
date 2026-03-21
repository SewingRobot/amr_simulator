export function EnvironmentRenderer() {
  return (
    <group>
      {/* 20x20 grid with 1m spacing */}
      <gridHelper args={[20, 20, '#444444', '#222222']} />
      {/* Axes: X=red, Y=green, Z=blue */}
      <axesHelper args={[5]} />
    </group>
  )
}
