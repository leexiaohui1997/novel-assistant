import React, { useEffect, useMemo, useRef, useState } from 'react'

import { useClipboard } from './hooks/useClipboard'
import { useShortcuts } from './hooks/useShortcuts'
import { dispatchInput } from './utils/dispatch'
import { toPlainText, parseDocument } from './utils/document'
import { createHistory, pushUndo } from './utils/history'
import { getSelectionFromDOM, setDOMSelection } from './utils/selection'

import type { HistoryState } from './utils/history'
import type { EditorSelection } from './utils/selection'

import { logger } from '@/utils/logger'

interface EditorProps {
  value?: string
  className?: string
  onChange?: (value: string) => void
  /** 文档为空时显示的占位内容 */
  placeholder?: React.ReactNode
}

const Editor: React.FC<EditorProps> = ({
  value = '',
  className = '',
  placeholder = '',
  onChange,
}) => {
  const editorRef = useRef<HTMLDivElement>(null)
  const [history, setHistory] = useState<HistoryState>(() => {
    const doc = parseDocument(value)
    return createHistory({
      doc,
      selection: {
        anchor: { nodeId: doc[0].id, offset: 0 },
        focus: { nodeId: doc[0].id, offset: 0 },
      },
    })
  })
  const pendingSelectionRef = useRef<EditorSelection | null>(null)

  const document = history.present.doc
  const plainText = useMemo(() => toPlainText(document).trim(), [document])

  // 文档内容变更时通知外部
  const onChangeRef = useRef(onChange)
  useEffect(() => {
    onChangeRef.current = onChange
  })
  useEffect(() => {
    if (plainText !== value) {
      onChangeRef.current?.(plainText)
    }
  }, [plainText, value])

  // React 渲染后恢复光标
  useEffect(() => {
    if (pendingSelectionRef.current && editorRef.current) {
      setDOMSelection(editorRef.current, pendingSelectionRef.current)
      pendingSelectionRef.current = null
    }
  })

  // 监听原生 beforeinput 事件（React 合成事件丢失 inputType）
  useEffect(() => {
    const el = editorRef.current
    if (!el) return

    const handler = (e: InputEvent) => {
      e.preventDefault()
      const sel = getSelectionFromDOM(el)
      if (!sel) return

      const { inputType, data } = e
      const result = dispatchInput(document, sel, inputType, data)

      if (result) {
        setHistory((prev) => pushUndo(prev, { doc: result.doc, selection: result.nextSelection }))
        pendingSelectionRef.current = result.nextSelection
      } else {
        logger.debug('不支持的 inputType:', inputType)
      }
    }

    el.addEventListener('beforeinput', handler)
    return () => el.removeEventListener('beforeinput', handler)
  }, [document])

  useClipboard({ editorRef, document, setHistory, pendingSelectionRef })

  const handleKeyDown = useShortcuts({ setHistory, pendingSelectionRef })

  const renderedContent = useMemo(() => {
    return document.map((p) => (
      <p key={p.id} className="indent-[2em] mb-4!" data-node-id={p.id}>
        {p.text || <br />}
      </p>
    ))
  }, [document])

  const isEmpty = document.length === 1 && document[0].text === ''

  return (
    <div className={`${className} relative text-base`}>
      <div
        ref={editorRef}
        className="outline-none p-2!"
        contentEditable
        suppressContentEditableWarning
        onKeyDown={handleKeyDown}
      >
        {renderedContent}
      </div>
      {isEmpty && placeholder && (
        <div className="pointer-events-none absolute inset-0 p-2! select-none text-[#bfbfbf]">
          <span className="pl-[2em]!"></span>
          {placeholder}
        </div>
      )}
    </div>
  )
}

export default Editor
