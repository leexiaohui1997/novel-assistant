import { useEffect } from 'react'

import { getSelectedText, pasteText } from '../utils/clipboard'
import { pushUndo } from '../utils/history'
import { deleteRange } from '../utils/operations'
import { getSelectionFromDOM } from '../utils/selection'

import type { Paragraph } from '../utils/document'
import type { HistoryState } from '../utils/history'
import type { EditorSelection } from '../utils/selection'

interface UseClipboardOptions {
  editorRef: React.RefObject<HTMLDivElement | null>
  document: Paragraph[]
  setHistory: React.Dispatch<React.SetStateAction<HistoryState>>
  pendingSelectionRef: React.MutableRefObject<EditorSelection | null>
}

/**
 * 剪贴板事件 hook
 *
 * 监听原生 copy / cut / paste 事件，
 * 拦截浏览器默认行为，走编辑器自定义数据流。
 */
export function useClipboard({
  editorRef,
  document,
  setHistory,
  pendingSelectionRef,
}: UseClipboardOptions) {
  useEffect(() => {
    const el = editorRef.current
    if (!el) return

    const onCopy = (e: ClipboardEvent) => {
      const sel = getSelectionFromDOM(el)
      if (!sel) return
      const text = getSelectedText(document, sel)
      if (!text) return
      e.preventDefault()
      e.clipboardData?.setData('text/plain', text)
    }

    const onCut = (e: ClipboardEvent) => {
      const sel = getSelectionFromDOM(el)
      if (!sel) return
      const text = getSelectedText(document, sel)
      if (!text) return
      e.preventDefault()
      e.clipboardData?.setData('text/plain', text)

      const result = deleteRange(document, sel)
      if (result) {
        setHistory((prev) => pushUndo(prev, { doc: result.doc, selection: result.nextSelection }))
        pendingSelectionRef.current = result.nextSelection
      }
    }

    const onPaste = (e: ClipboardEvent) => {
      const text = e.clipboardData?.getData('text/plain')
      if (!text) return
      e.preventDefault()

      const sel = getSelectionFromDOM(el)
      if (!sel) return

      const result = pasteText(document, sel, text)
      if (result) {
        setHistory((prev) => pushUndo(prev, { doc: result.doc, selection: result.nextSelection }))
        pendingSelectionRef.current = result.nextSelection
      }
    }

    el.addEventListener('copy', onCopy)
    el.addEventListener('cut', onCut)
    el.addEventListener('paste', onPaste)
    return () => {
      el.removeEventListener('copy', onCopy)
      el.removeEventListener('cut', onCut)
      el.removeEventListener('paste', onPaste)
    }
  }, [document, editorRef, setHistory, pendingSelectionRef])
}
