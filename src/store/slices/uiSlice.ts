import { createSlice, PayloadAction } from '@reduxjs/toolkit'

type UITheme = 'light' | 'dark'

/**
 * UI 状态接口
 * 管理应用界面的全局状态
 */
interface UIState {
  /** 主题模式：浅色或深色 */
  theme: UITheme
  /** 侧边栏是否折叠 */
  sidebarCollapsed: boolean
  /** 当前页面标识 */
  currentPage: string
}

const initialState: UIState = {
  theme: 'light',
  sidebarCollapsed: false,
  currentPage: 'home',
}

const uiSlice = createSlice({
  name: 'ui',
  initialState,
  reducers: {
    /**
     * 设置应用主题
     * @param state - 当前 UI 状态
     * @param action - 包含主题值的 payload（'light' 或 'dark'）
     */
    setTheme: (state, action: PayloadAction<UITheme>) => {
      state.theme = action.payload
    },
    /**
     * 切换侧边栏折叠状态
     * @param state - 当前 UI 状态
     */
    toggleSidebar: (state) => {
      state.sidebarCollapsed = !state.sidebarCollapsed
    },
    /**
     * 设置当前页面
     * @param state - 当前 UI 状态
     * @param action - 包含页面标识的 payload
     */
    setCurrentPage: (state, action: PayloadAction<string>) => {
      state.currentPage = action.payload
    },
  },
})

export const { setTheme, toggleSidebar, setCurrentPage } = uiSlice.actions
export default uiSlice.reducer
