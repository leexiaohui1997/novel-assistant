import { Menu, type MenuProps } from 'antd'
import { last } from 'lodash-es'
import { useEffect, useMemo } from 'react'
import { useNavigate, useLocation, useMatches } from 'react-router-dom'

import type { MenuItem } from '@/config'

import { useRouteMeta } from '@/hooks/useRouteMeta'
import { useAppSelector, useAppDispatch } from '@/store/hooks'
import { setActiveMenuKey } from '@/store/slices/uiSlice'
import { deepFindArr, deepFindArrWithPath } from '@/utils/array'

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
  const { menuItems } = useAppSelector((state) => state.ui)
  const matches = useMatches()
  const routeMeta = useRouteMeta()

  const activeMenuKey = useMemo(() => {
    return routeMeta.activeMenuKey || last(matches)?.id || ''
  }, [matches, routeMeta])

  const defaultOpenKeys = useMemo(() => {
    return deepFindArrWithPath(
      menuItems,
      (item) => item.children?.some((child) => child.key === activeMenuKey) || false,
    ).map((item) => item.key)
  }, [menuItems, activeMenuKey])

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
    if (foundItem?.path) {
      navigate(foundItem.path)
    }
  }

  return (
    <aside className="sidebar">
      <div className="sidebar-inner">
        <Menu
          mode="inline"
          selectedKeys={[activeMenuKey]}
          defaultOpenKeys={defaultOpenKeys}
          items={convertMenuItems(menuItems)}
          onClick={handleMenuClick}
          style={{ borderRight: 'none' }}
        />
      </div>
    </aside>
  )
}

export default Sidebar
