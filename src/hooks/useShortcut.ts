import React, { useCallback } from 'react'
import { useSlate } from 'slate-react'

import { getShortcutText } from '@/utils/keyboard'

enum ShortcutType {
  Undo = 'undo',
  Redo = 'redo',
}

type ShortCutHandleCtx = {
  editor?: ReturnType<typeof useSlate>
}

type ShortCutItemConfig = {
  type: ShortcutType
  keys: string[]
  handler: (ctx: ShortCutHandleCtx) => unknown
}

type UseShortcutProps = {
  enable?: Record<ShortcutType, boolean>
} & ShortCutHandleCtx

const shortcutItems: ShortCutItemConfig[] = [
  {
    type: ShortcutType.Undo,
    keys: ['Ctrl + Z', 'Meta + Z'],
    handler: ({ editor }) => {
      editor?.undo()
    },
  },
  {
    type: ShortcutType.Redo,
    keys: ['Ctrl + Y', 'Meta + Y', 'Ctrl + Shift + Z', 'Meta + Shift + Z'],
    handler: ({ editor }) => {
      editor?.redo()
    },
  },
]

export const useShortcut = ({ enable, editor }: UseShortcutProps) => {
  const onKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const key = getShortcutText(e)
      const item = shortcutItems.find((item) => item.keys.includes(key))
      if (item && (enable?.[item.type] ?? true)) {
        e.preventDefault()
        item.handler({ editor })
      }
    },
    [editor, enable],
  )

  return [onKeyDown] as const
}
