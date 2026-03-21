import type { ReactNode } from 'react'

type Page = 'viewer' | 'dashboard'

interface LayoutProps {
  children: ReactNode
  currentPage: Page
  onNavigate: (page: Page) => void
}

const navItems: { page: Page; label: string }[] = [
  { page: 'viewer', label: '3D Viewer' },
  { page: 'dashboard', label: 'Dashboard' },
]

export function Layout({ children, currentPage, onNavigate }: LayoutProps) {
  return (
    <div className="flex h-screen w-screen bg-gray-900 text-white">
      {/* Sidebar */}
      <aside className="w-56 flex-shrink-0 bg-gray-950 border-r border-gray-800 flex flex-col">
        <div className="p-4 border-b border-gray-800">
          <h1 className="text-lg font-bold tracking-tight">AMR Simulator</h1>
        </div>
        <nav className="flex-1 p-2 space-y-1">
          {navItems.map(({ page, label }) => (
            <button
              key={page}
              onClick={() => onNavigate(page)}
              className={`w-full text-left px-3 py-2 rounded text-sm transition-colors ${
                currentPage === page
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-400 hover:bg-gray-800 hover:text-white'
              }`}
            >
              {label}
            </button>
          ))}
        </nav>
        <div className="p-4 border-t border-gray-800 text-xs text-gray-500">
          v0.1.0 Prototype
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* Top bar */}
        <header className="h-12 flex-shrink-0 bg-gray-950 border-b border-gray-800 flex items-center px-4">
          <span className="text-sm text-gray-400">
            {navItems.find((n) => n.page === currentPage)?.label}
          </span>
        </header>

        {/* Content area */}
        <div className="flex-1 overflow-hidden">{children}</div>
      </main>
    </div>
  )
}
