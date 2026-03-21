import { useState, useEffect, useRef } from 'react'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'
import type { Group } from 'three'

interface UseModelLoaderResult {
  scene: Group | null
  isLoading: boolean
  error: string | null
}

/**
 * Attempts to load a glTF model from `/api/assets/{modelId}/download`.
 * Returns the loaded scene, or null if loading fails (caller uses fallback box).
 */
export function useModelLoader(modelId: string | undefined): UseModelLoaderResult {
  const [scene, setScene] = useState<Group | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const loaderRef = useRef<GLTFLoader | null>(null)

  useEffect(() => {
    if (!modelId) {
      setScene(null)
      setIsLoading(false)
      setError(null)
      return
    }

    if (!loaderRef.current) {
      loaderRef.current = new GLTFLoader()
    }

    let cancelled = false
    setIsLoading(true)
    setError(null)

    const url = `/api/assets/${encodeURIComponent(modelId)}/download`

    loaderRef.current.load(
      url,
      (gltf) => {
        if (!cancelled) {
          setScene(gltf.scene)
          setIsLoading(false)
        }
      },
      undefined,
      (err) => {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : 'Failed to load model'
          setError(message)
          setScene(null)
          setIsLoading(false)
        }
      },
    )

    return () => {
      cancelled = true
    }
  }, [modelId])

  return { scene, isLoading, error }
}
