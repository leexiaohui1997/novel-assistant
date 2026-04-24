import React, { useCallback, useMemo, useState } from 'react'

import { getShortcutText } from '@/utils/keyboard'
import { logger } from '@/utils/logger'

interface EditorProps {
  value?: string
  className?: string
}

const Editor: React.FC<EditorProps> = ({ value = '222', className = '' }) => {
  const [innerValue] = useState(value)
  const contentParts = useMemo(() => innerValue.split('\n'), [innerValue])

  const handleBeforeInput = useCallback((e: React.InputEvent) => {
    e.preventDefault()
    const selection = window.getSelection()
    console.log({ e, selection })
  }, [])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    logger.debug(getShortcutText(e))
  }, [])

  const renderedContent = useMemo(() => {
    return contentParts.map((part, index) => <p key={index}>{part}</p>)
  }, [contentParts])

  return (
    <div
      className={`${className} outline-none`}
      contentEditable
      suppressContentEditableWarning
      onBeforeInput={handleBeforeInput}
      onKeyDown={handleKeyDown}
    >
      {renderedContent}
    </div>
  )
}

export default Editor
