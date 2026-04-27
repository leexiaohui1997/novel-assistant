import { Menu } from 'antd'
import React, { useMemo } from 'react'
import { Outlet, useMatches } from 'react-router-dom'

const AISettings: React.FC = () => {
  const matches = useMatches()
  const selectedKeys = useMemo(() => {
    return matches.map((match) => match.id)
  }, [matches])

  return (
    <>
      <Menu
        mode="horizontal"
        className="bg-transparent! mb-4!"
        selectedKeys={selectedKeys}
        items={[
          {
            key: 'ai-settings-models',
            label: '模型管理',
          },
        ]}
      />

      <div className="flex-1 flex flex-col">
        <Outlet />
      </div>
    </>
  )
}

export default AISettings
