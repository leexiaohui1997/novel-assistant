/**
 * 菜单项接口定义
 */
export interface MenuItem {
  /** 菜单唯一标识 */
  key: string
  /** 菜单显示标签 */
  label: string
  /** 路由路径 */
  path?: string
  /** 子菜单项 */
  children?: MenuItem[]
}

/**
 * 默认菜单配置
 */
export const DEFAULT_MENU_ITEMS: MenuItem[] = [
  {
    key: 'novels',
    label: '作品管理',
    path: '/novels',
  },
  {
    key: 'settings',
    label: '设置',
    children: [
      {
        key: 'ai-providers',
        label: '供应商管理',
        path: '/settings/providers',
      },
    ],
  },
]
