import { Canvas } from '@react-three/fiber'
import { EnvironmentRenderer } from './EnvironmentRenderer'
import { RobotManager } from './RobotManager'
import { CameraController } from './CameraController'
import { ViewModeToggle } from './ViewModeToggle'

export function SceneCanvas() {
  return (
    <div className="relative w-full h-full">
      <ViewModeToggle />
      <Canvas camera={{ position: [20, 15, 5], fov: 60 }}>
        <ambientLight intensity={0.5} />
        <directionalLight position={[10, 10, 5]} intensity={1} />
        <CameraController />
        <EnvironmentRenderer />
        <RobotManager />
      </Canvas>
    </div>
  )
}
