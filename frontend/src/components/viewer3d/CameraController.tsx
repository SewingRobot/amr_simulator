import { useEffect, useRef, useCallback } from 'react'
import { OrbitControls } from '@react-three/drei'
import { useThree } from '@react-three/fiber'
import type { OrbitControls as OrbitControlsImpl } from 'three-stdlib'
import { useViewerStore } from '../../stores/viewerStore'

type CameraMode = 'orbit' | 'topdown' | 'follow'

export function CameraController() {
  const controlsRef = useRef<OrbitControlsImpl>(null)
  const { camera } = useThree()
  const cameraMode = useViewerStore((state) => state.cameraMode)
  const setCameraMode = useViewerStore((state) => state.setCameraMode)

  const applyMode = useCallback(
    (mode: CameraMode) => {
      switch (mode) {
        case 'topdown':
          camera.position.set(0, 20, 0)
          camera.lookAt(0, 0, 0)
          if (controlsRef.current) {
            controlsRef.current.maxPolarAngle = 0
            controlsRef.current.minPolarAngle = 0
          }
          break
        case 'orbit':
        default:
          camera.position.set(10, 10, 10)
          camera.lookAt(0, 0, 0)
          if (controlsRef.current) {
            controlsRef.current.maxPolarAngle = Math.PI / 2
            controlsRef.current.minPolarAngle = 0
          }
          break
        case 'follow':
          // Follow mode will be updated per-frame in a later iteration
          break
      }
    },
    [camera],
  )

  useEffect(() => {
    applyMode(cameraMode)
  }, [cameraMode, applyMode])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case '1':
          setCameraMode('orbit')
          break
        case '2':
          setCameraMode('topdown')
          break
        case '3':
          setCameraMode('follow')
          break
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [setCameraMode])

  return (
    <OrbitControls
      ref={controlsRef}
      enableDamping
      dampingFactor={0.1}
      maxPolarAngle={Math.PI / 2}
    />
  )
}
