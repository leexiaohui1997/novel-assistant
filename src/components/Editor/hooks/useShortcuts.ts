import { useCallback, useMemo } from 'react'

import { redo, undo } from '../utils/history'

import type { HistoryState } from '../utils/history'
import type { EditorSelection } from '../utils/selection'

import { getShortcutText } from '@/utils/keyboard'
import { logger } from '@/utils/logger'

interface UseShortcutsOptions {
  setHistory: React.Dispatch<React.SetStateAction<HistoryState>>
  pendingSelectionRef: React.MutableRefObject<EditorSelection | null>
}

/**
 * 编辑器快捷键 hook
 *
 * 维护「快捷键名 → 操作函数」映射表，返回 handleKeyDown 供组件绑定。
 * 新增快捷键只需在 shortcutMap 中添加条目。
 */
export function useShortcuts({ setHistory, pendingSelectionRef }: UseShortcutsOptions) {
  const handleUndo = useCallback(() => {
    setHistory((prev) => {
      const next = undo(prev)
      if (next === prev) return prev
      pendingSelectionRef.current = next.present.selection
      return next
    })
  }, [setHistory, pendingSelectionRef])

  const handleRedo = useCallback(() => {
    setHistory((prev) => {
      const next = redo(prev)
      if (next === prev) return prev
      pendingSelectionRef.current = next.present.selection
      return next
    })
  }, [setHistory, pendingSelectionRef])

  /** 快捷键名 → 操作函数映射，新增快捷键只需在此添加条目 */
  const shortcutMap = useMemo<Record<string, () => void>>(
    () => ({
      'Ctrl + Z': handleUndo,
      'Ctrl + Shift + Z': handleRedo,
      'Ctrl + Y': handleRedo,
    }),
    [handleUndo, handleRedo],
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const shortcut = getShortcutText(e)
      const action = shortcutMap[shortcut]
      if (action) {
        e.preventDefault()
        action()
        return
      }
      logger.debug(shortcut)
    },
    [shortcutMap],
  )

  return handleKeyDown
}
