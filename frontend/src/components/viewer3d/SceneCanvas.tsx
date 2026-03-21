import { Canvas } from '@react-three/fiber'
import { EnvironmentRenderer } from './EnvironmentRenderer'
import { RobotManager } from './RobotManager'
import { CameraController } from './CameraController'

export function SceneCanvas() {
  return (
    <Canvas camera={{ position: [10, 10, 10], fov: 60 }}>
      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 10, 5]} intensity={1} />
      <CameraController />
      <EnvironmentRenderer />
      <RobotManager />
    </Canvas>
  )
}
