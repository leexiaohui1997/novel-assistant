import { Suspense } from 'react'
import { Outlet, useMatches } from 'react-router-dom'

import Header from './Header'
import Sidebar from './Sidebar'

import type { RouteHandle } from '@/config'

import PageLoading from '@/components/PageLoading'

import './styles.css'

const Layout: React.FC = () => {
  const matches = useMatches()
  const currentMatch = matches[matches.length - 1]
  const handle = currentMatch?.handle as RouteHandle | undefined
  const hideSidebar = handle?.hideSidebar ?? false
  const layoutClassNames = handle?.layoutClassNames ?? {}

  return (
    <div className={`layout-container${hideSidebar ? ' layout-container--no-sidebar' : ''}`}>
      <Header />
      <div className={`main-wrapper ${layoutClassNames.wrapper}`}>
        {!hideSidebar && <Sidebar />}
        <main className="main-content">
          <Suspense fallback={<PageLoading />}>
            <Outlet />
          </Suspense>
        </main>
      </div>
    </div>
  )
}

export default Layout
