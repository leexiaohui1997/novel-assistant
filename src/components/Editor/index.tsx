import React, { useCallback, useMemo, useState } from 'react'
import { createEditor, Descendant, Node } from 'slate'
import { Slate, Editable, withReact, useSlateSelector } from 'slate-react'

import { deserializeValue, initialValue, serializeValue } from './utils/document'

interface EditorProps {
  value?: string
  className?: string
  placeholder?: React.ReactNode
  onChange?: (value: string) => void
}

const EditorContent: React.FC<EditorProps> = ({ className, placeholder }) => {
  // 跟踪组合输入状态
  const [isComposing, setIsComposing] = useState(false)

  // 使用 useSlateSelector 监听编辑器内容变化，判断是否显示 placeholder
  const showPlaceholder = useSlateSelector((editor) => {
    if (!placeholder) return false

    // 如果编辑器正在组合输入（中文输入法状态），不显示 placeholder
    if (isComposing) return false

    // 检查编辑器内容是否为空
    const isEmpty = editor.children.every((node) => {
      const text = Node.string(node)
      return !text || text.trim() === ''
    })
    return isEmpty
  })

  return (
    <div className="flex-1 flex flex-col relative text-base indent-[2em]">
      {showPlaceholder && (
        <div className="pointer-events-none absolute inset-0 py-2! px-2.75!">
          <div className="text-[rgba(0,0,0,0.25)]">{placeholder}</div>
        </div>
      )}
      <Editable
        className={`py-2! px-2.75! flex-1 flex flex-col gap-4 outline-none ${className}`}
        onCompositionStart={() => setIsComposing(true)}
        onCompositionEnd={() => setIsComposing(false)}
      />
    </div>
  )
}

const Editor: React.FC<EditorProps> = (props) => {
  const { value, onChange } = props
  const editor = useMemo(() => withReact(createEditor()), [])

  // 根据 value prop 计算编辑器初始值
  const editorValue = useMemo(() => {
    return value !== undefined ? deserializeValue(value) : initialValue
  }, [value])

  // 处理编辑器内容变化
  const handleChange = useCallback(
    (newValue: Descendant[]) => {
      if (onChange) {
        const textValue = serializeValue(newValue)
        onChange(textValue)
      }
    },
    [onChange],
  )

  return (
    <Slate editor={editor} initialValue={editorValue} onChange={handleChange}>
      <EditorContent {...props} />
    </Slate>
  )
}

export default Editor
