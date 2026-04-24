import React, { useEffect, useMemo, useRef, useState } from 'react'

import { dispatchInput } from './utils/dispatch'
import { parseDocument } from './utils/document'
import { getSelectionFromDOM, setDOMSelection } from './utils/selection'

import type { Paragraph } from './utils/document'
import type { EditorSelection } from './utils/selection'

import { getShortcutText } from '@/utils/keyboard'
import { logger } from '@/utils/logger'

interface EditorProps {
  value?: string
  className?: string
}

const Editor: React.FC<EditorProps> = ({ value = '', className = '' }) => {
  const editorRef = useRef<HTMLDivElement>(null)
  const [document, setDocument] = useState<Paragraph[]>(() => parseDocument(value))
  const pendingSelectionRef = useRef<EditorSelection | null>(null)

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
        setDocument(result.doc)
        pendingSelectionRef.current = result.nextSelection
      } else {
        logger.debug('不支持的 inputType:', inputType)
      }
    }

    el.addEventListener('beforeinput', handler)
    return () => el.removeEventListener('beforeinput', handler)
  }, [document])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    logger.debug(getShortcutText(e))
  }

  const renderedContent = useMemo(() => {
    return document.map((p) => (
      <p key={p.id} data-node-id={p.id}>
        {p.text || <br />}
      </p>
    ))
  }, [document])

  return (
    <div
      ref={editorRef}
      className={`${className} outline-none`}
      contentEditable
      suppressContentEditableWarning
      onKeyDown={handleKeyDown}
    >
      {renderedContent}
    </div>
  )
}

export default Editor
