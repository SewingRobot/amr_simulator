import { useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import {
  Quaternion as ThreeQuaternion,
  Vector3 as ThreeVector3,
} from 'three'
import type { Group } from 'three'
import { enuToThreeJS, enuQuaternionToThreeJS } from '../../utils/math'
import { useModelLoader } from '../../hooks/useModelLoader'
import type { Pose } from '../../types/robot'

interface RobotModelProps {
  pose: Pose
  selected?: boolean
  color?: string
  name?: string
  modelId?: string
  onClick?: () => void
}

export function RobotModel({
  pose,
  selected = false,
  color = '#3b82f6',
  name,
  modelId,
  onClick,
}: RobotModelProps) {
  const { scene: gltfScene } = useModelLoader(modelId)
  const groupRef = useRef<Group>(null)
  const targetPos = useRef(new ThreeVector3())
  const targetQuat = useRef(new ThreeQuaternion())
  const initialized = useRef(false)

  // Update target pose every render (driven by store changes)
  const [tx, ty, tz] = enuToThreeJS(
    pose.position.x,
    pose.position.y,
    pose.position.z,
  )
  targetPos.current.set(tx, ty, tz)

  const tq = enuQuaternionToThreeJS(pose.orientation)
  targetQuat.current.set(tq.x, tq.y, tq.z, tq.w)

  useFrame((_, delta) => {
    if (!groupRef.current) return

    if (!initialized.current) {
      // Snap to initial position on first frame
      groupRef.current.position.copy(targetPos.current)
      groupRef.current.quaternion.copy(targetQuat.current)
      initialized.current = true
      return
    }

    // Smooth interpolation: lerp position, slerp rotation
    const lerpFactor = Math.min(1, delta * 15)
    groupRef.current.position.lerp(targetPos.current, lerpFactor)
    groupRef.current.quaternion.slerp(targetQuat.current, lerpFactor)
  })

  return (
    <group ref={groupRef} onClick={onClick}>
      {/* Use glTF model if loaded, otherwise fallback to box */}
      {gltfScene ? (
        <primitive object={gltfScene.clone()} />
      ) : (
        <>
          {/* Robot body: 0.5m x 0.3m x 0.2m */}
          <mesh>
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
        </>
      )}

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
