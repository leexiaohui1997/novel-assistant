import { Menu, type MenuProps } from 'antd'
import { useEffect } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'

import type { MenuItem } from '@/config'

import { useAppSelector, useAppDispatch } from '@/store/hooks'
import { setActiveMenuKey } from '@/store/slices/uiSlice'
import { deepFindArr } from '@/utils/array'

import './styles.css'

/**
 * 将 MenuItem 转换为 Ant Design Menu 需要的格式
 */
const convertMenuItems = (items: MenuItem[]): MenuProps['items'] => {
  return items.map((item) => ({
    key: item.key,
    label: item.label,
    children: item.children ? convertMenuItems(item.children) : undefined,
  }))
}

const Sidebar: React.FC = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const dispatch = useAppDispatch()
  const { menuItems, activeMenuKey } = useAppSelector((state) => state.ui)

  // 监听路由变化，同步菜单高亮状态
  useEffect(() => {
    const foundItem = deepFindArr<MenuItem>(menuItems, (item) => item.path === location.pathname)
    if (foundItem && foundItem.key !== activeMenuKey) {
      dispatch(setActiveMenuKey(foundItem.key))
    }
  }, [location.pathname, menuItems, activeMenuKey, dispatch])

  const handleMenuClick: MenuProps['onClick'] = ({ key }) => {
    // 查找对应的菜单项获取路径
    const foundItem = deepFindArr<MenuItem>(menuItems, (item) => item.key === key)
    if (foundItem) {
      navigate(foundItem.path)
      dispatch(setActiveMenuKey(key))
    }
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-inner">
        <Menu
          mode="inline"
          selectedKeys={[activeMenuKey]}
          items={convertMenuItems(menuItems)}
          onClick={handleMenuClick}
          style={{ borderRight: 'none' }}
        />
      </div>
    </aside>
  )
}

export default Sidebar
