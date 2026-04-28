import React, {
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from 'react'
import { createEditor, Descendant, Editor as SlateEditor, Node, Transforms } from 'slate'
import { Slate, Editable, withReact, useSlateSelector } from 'slate-react'

import { deserializeValue, initialValue, serializeValue } from './utils/document'

interface EditorProps {
  value?: string
  className?: string
  placeholder?: React.ReactNode
  onChange?: (value: string) => void
}

/** Editor 对外暴露的命令式方法 */
export interface EditorHandle {
  /** 替换编辑器内容（选中全部 → 插入替换） */
  setContent: (content: string) => void
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

const Editor = React.forwardRef<EditorHandle, EditorProps>((props, ref) => {
  const { value, onChange } = props
  const editor = useMemo(() => withReact(createEditor()), [])
  const onChangeRef = useRef(onChange)
  useEffect(() => {
    onChangeRef.current = onChange
  })

  // 对外暴露 setContent：选中全部 → 插入替换
  useImperativeHandle(ref, () => ({
    setContent: (content: string) => {
      const nodes = deserializeValue(content)
      // 先删除全部节点
      Transforms.delete(editor, {
        at: {
          anchor: SlateEditor.start(editor, []),
          focus: SlateEditor.end(editor, []),
        },
      })
      // 再插入新节点
      Transforms.insertNodes(editor, nodes, { at: [0], select: true })
    },
  }))

  // 根据 value prop 计算编辑器初始值
  const editorValue = useMemo(() => {
    return value !== undefined ? deserializeValue(value) : initialValue
  }, [value])

  // 处理编辑器内容变化
  const handleChange = useCallback((newValue: Descendant[]) => {
    if (onChangeRef.current) {
      const textValue = serializeValue(newValue)
      onChangeRef.current(textValue)
    }
  }, [])

  return (
    <Slate editor={editor} initialValue={editorValue} onChange={handleChange}>
      <EditorContent {...props} />
    </Slate>
  )
})

export default Editor
