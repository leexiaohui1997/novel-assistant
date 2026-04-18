import { Suspense } from 'react'
import { Routes, Route } from 'react-router-dom'

import { Layout } from '@/components/layout'
import PageLoading from '@/components/PageLoading'
import { ROUTES_CONFIG } from '@/config'

const App: React.FC = () => {
  return (
    <Layout>
      <Suspense fallback={<PageLoading />}>
        <Routes>
          {ROUTES_CONFIG.map((route) => (
            <Route key={route.path} path={route.path} element={<route.component />} />
          ))}
        </Routes>
      </Suspense>
    </Layout>
  )
}

export default App
