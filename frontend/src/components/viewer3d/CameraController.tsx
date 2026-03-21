import { useEffect, useRef, useCallback } from 'react'
import { OrbitControls } from '@react-three/drei'
import { useThree, useFrame } from '@react-three/fiber'
import * as THREE from 'three'
import type { OrbitControls as OrbitControlsImpl } from 'three-stdlib'
import { useViewerStore } from '../../stores/viewerStore'

type CameraMode = 'orbit' | 'topdown' | 'follow'

export function CameraController() {
  const controlsRef = useRef<OrbitControlsImpl>(null)
  const { camera } = useThree()
  const cameraMode = useViewerStore((state) => state.cameraMode)
  const setCameraMode = useViewerStore((state) => state.setCameraMode)
  const viewMode = useViewerStore((state) => state.viewMode)
  const toggleViewMode = useViewerStore((state) => state.toggleViewMode)

  // Animation state for smooth camera transitions
  const animating = useRef(false)
  const animTarget = useRef({ position: new THREE.Vector3(), zoom: 1 })
  const animProgress = useRef(0)
  const animStart = useRef({ position: new THREE.Vector3(), zoom: 1 })

  const applyMode = useCallback(
    (mode: CameraMode) => {
      // In 2D mode, camera mode changes are ignored (always top-down ortho)
      if (viewMode === '2d') return

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
    [camera, viewMode],
  )

  // Handle view mode transitions
  useEffect(() => {
    if (viewMode === '2d') {
      // Animate to top-down orthographic position
      animStart.current.position.copy(camera.position)
      animStart.current.zoom = (camera as THREE.PerspectiveCamera).zoom ?? 1
      animTarget.current.position.set(0, 50, 0.001) // slight Z offset to avoid gimbal lock
      animTarget.current.zoom = 1
      animProgress.current = 0
      animating.current = true

      if (controlsRef.current) {
        controlsRef.current.enableRotate = false
        controlsRef.current.maxPolarAngle = 0
        controlsRef.current.minPolarAngle = 0
      }
    } else {
      // Animate back to 3D perspective
      animStart.current.position.copy(camera.position)
      animTarget.current.position.set(10, 10, 10)
      animProgress.current = 0
      animating.current = true

      if (controlsRef.current) {
        controlsRef.current.enableRotate = true
        controlsRef.current.maxPolarAngle = Math.PI / 2
        controlsRef.current.minPolarAngle = 0
      }
    }
  }, [viewMode, camera])

  // Animate camera position each frame
  useFrame((_, delta) => {
    if (!animating.current) return

    animProgress.current += delta * 3 // speed factor
    const t = Math.min(animProgress.current, 1)
    // Smooth ease-in-out
    const eased = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2

    camera.position.lerpVectors(
      animStart.current.position,
      animTarget.current.position,
      eased,
    )
    camera.lookAt(0, 0, 0)

    if (controlsRef.current) {
      controlsRef.current.target.set(0, 0, 0)
      controlsRef.current.update()
    }

    if (t >= 1) {
      animating.current = false
    }
  })

  useEffect(() => {
    applyMode(cameraMode)
  }, [cameraMode, applyMode])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore key events when typing in inputs
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      )
        return

      switch (e.key.toLowerCase()) {
        case 'v':
          toggleViewMode()
          break
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
  }, [setCameraMode, toggleViewMode])

  return (
    <OrbitControls
      ref={controlsRef}
      enableDamping
      dampingFactor={0.1}
      enableRotate={viewMode === '3d'}
      maxPolarAngle={viewMode === '2d' ? 0 : Math.PI / 2}
      minPolarAngle={0}
    />
  )
}
