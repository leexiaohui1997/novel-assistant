import React from 'react'

interface EditorProps {
  value?: string
  className?: string
  placeholder?: React.ReactNode
  onChange?: (value: string) => void
}

const Editor: React.FC<EditorProps> = () => {
  return <div>Editor</div>
}

export default Editor
