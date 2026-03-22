import { useState } from 'react'
import { Layout } from './components/common/Layout'
import { ConnectionStatus } from './components/common/ConnectionStatus'
import { SceneCanvas } from './components/viewer3d/SceneCanvas'
import { RobotInfoPanel } from './components/viewer3d/RobotInfoPanel'
import { DashboardPage } from './components/dashboard/DashboardPage'
import { MissionListPage } from './components/missions/MissionListPage'

type Page = 'viewer' | 'dashboard' | 'missions'

function App() {
  const [page, setPage] = useState<Page>('viewer')

  return (
    <Layout currentPage={page} onNavigate={setPage}>
      <ConnectionStatus />
      {page === 'viewer' && (
        <div className="relative w-full h-full">
          <SceneCanvas />
          <RobotInfoPanel />
        </div>
      )}
      {page === 'dashboard' && <DashboardPage />}
      {page === 'missions' && <MissionListPage />}
    </Layout>
  )
}

export default App
