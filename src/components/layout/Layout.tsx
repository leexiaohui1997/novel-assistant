import React from 'react'

import Header from './Header'
import Sidebar from './Sidebar'

import './styles.css'

interface LayoutProps {
  children?: React.ReactNode
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  return (
    <div className="layout-container">
      <Header />
      <div className="main-wrapper">
        <Sidebar />
        <main className="main-content">{children}</main>
      </div>
    </div>
  )
}

export default Layout
