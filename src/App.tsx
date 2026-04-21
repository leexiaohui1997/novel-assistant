import { createBrowserRouter, RouterProvider } from 'react-router-dom'

import { ROUTES_CONFIG } from '@/config'

const router = createBrowserRouter(ROUTES_CONFIG)

const App: React.FC = () => {
  return <RouterProvider router={router} />
}

export default App
