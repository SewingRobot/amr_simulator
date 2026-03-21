import { useViewerStore } from '../../stores/viewerStore'

export function ViewModeToggle() {
  const viewMode = useViewerStore((state) => state.viewMode)
  const toggleViewMode = useViewerStore((state) => state.toggleViewMode)

  return (
    <button
      onClick={toggleViewMode}
      className="absolute top-3 left-3 z-10 flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-gray-900/70 hover:bg-gray-900/90 backdrop-blur-sm border border-gray-700 text-sm font-medium text-white transition-colors select-none"
      title={`Switch to ${viewMode === '3d' ? '2D' : '3D'} view (V)`}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        {viewMode === '3d' ? (
          /* 3D cube icon */
          <>
            <path d="M12 2L2 7l10 5 10-5-10-5z" />
            <path d="M2 17l10 5 10-5" />
            <path d="M2 12l10 5 10-5" />
          </>
        ) : (
          /* 2D square icon */
          <rect x="3" y="3" width="18" height="18" rx="2" />
        )}
      </svg>
      <span>{viewMode.toUpperCase()}</span>
    </button>
  )
}
