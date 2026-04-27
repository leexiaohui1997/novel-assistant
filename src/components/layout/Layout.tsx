import { Suspense } from 'react'
import { Outlet } from 'react-router-dom'

import Header from './Header'
import Sidebar from './Sidebar'

import PageLoading from '@/components/PageLoading'
import { useRouteMeta } from '@/hooks/useRouteMeta'
import './styles.css'

const Layout: React.FC = () => {
  const handle = useRouteMeta()
  const hideSidebar = handle?.hideSidebar ?? false
  const layoutClassNames = handle?.layoutClassNames ?? {}

  return (
    <div className={`layout-container${hideSidebar ? ' layout-container--no-sidebar' : ''}`}>
      <Header />
      <div className={`main-wrapper ${layoutClassNames.wrapper}`}>
        {!hideSidebar && <Sidebar />}
        <main className="main-content flex flex-col">
          <Suspense fallback={<PageLoading />}>
            <Outlet />
          </Suspense>
        </main>
      </div>
    </div>
  )
}

export default Layout
