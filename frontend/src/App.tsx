import { useState } from 'react'
import { Layout } from './components/common/Layout'
import { SceneCanvas } from './components/viewer3d/SceneCanvas'
import { RobotInfoPanel } from './components/viewer3d/RobotInfoPanel'
import { DashboardPage } from './components/dashboard/DashboardPage'

type Page = 'viewer' | 'dashboard'

function App() {
  const [page, setPage] = useState<Page>('viewer')

  return (
    <Layout currentPage={page} onNavigate={setPage}>
      {page === 'viewer' && (
        <div className="relative w-full h-full">
          <SceneCanvas />
          <RobotInfoPanel />
        </div>
      )}
      {page === 'dashboard' && <DashboardPage />}
    </Layout>
  )
}

export default App
