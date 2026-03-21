import { useRef } from 'react'
import type { Mesh } from 'three'
import { Euler, Quaternion } from 'three'
import { enuToThreeJS, enuQuaternionToThreeJS } from '../../utils/math'
import type { Pose } from '../../types/robot'

interface RobotModelProps {
  pose: Pose
  selected?: boolean
  color?: string
  name?: string
  onClick?: () => void
}

export function RobotModel({
  pose,
  selected = false,
  color = '#3b82f6',
  name,
  onClick,
}: RobotModelProps) {
  const meshRef = useRef<Mesh>(null)

  // Convert ENU pose to Three.js coordinates
  const position = enuToThreeJS(
    pose.position.x,
    pose.position.y,
    pose.position.z,
  )

  const threeQuat = enuQuaternionToThreeJS(pose.orientation)
  const euler = new Euler().setFromQuaternion(
    new Quaternion(threeQuat.x, threeQuat.y, threeQuat.z, threeQuat.w),
  )

  return (
    <group position={position} rotation={euler} onClick={onClick}>
      {/* Robot body: 0.5m x 0.3m x 0.2m */}
      <mesh ref={meshRef}>
        <boxGeometry args={[0.5, 0.2, 0.3]} />
        <meshStandardMaterial
          color={selected ? '#facc15' : color}
          emissive={selected ? '#facc15' : '#000000'}
          emissiveIntensity={selected ? 0.3 : 0}
        />
      </mesh>

      {/* Direction arrow (cone at front) */}
      <mesh position={[0.35, 0.05, 0]} rotation={[0, 0, -Math.PI / 2]}>
        <coneGeometry args={[0.06, 0.15, 8]} />
        <meshStandardMaterial color="#ef4444" />
      </mesh>

      {/* Selection ring */}
      {selected && (
        <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.1, 0]}>
          <ringGeometry args={[0.35, 0.4, 32]} />
          <meshBasicMaterial color="#facc15" transparent opacity={0.6} />
        </mesh>
      )}

      {/* Name label */}
      {name && (
        <sprite position={[0, 0.4, 0]} scale={[1, 0.3, 1]}>
          <spriteMaterial color="#ffffff" transparent opacity={0.8} />
        </sprite>
      )}
    </group>
  )
}
