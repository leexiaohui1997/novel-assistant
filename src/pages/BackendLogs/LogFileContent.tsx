import { useEffect } from 'react'

import { useLogContent } from './hooks/useLogContent'

import { MarkdownContent } from '@/components/Markdown/MarkdownContent'

export type LogFileContentProps = {
  filename?: string
}

export function LogFileContent({ filename }: LogFileContentProps) {
  const { content, loadContent } = useLogContent()

  useEffect(() => {
    if (filename) {
      loadContent(filename)
    }
  }, [filename, loadContent])

  return (
    <div>
      <MarkdownContent content={'```log\n' + content?.trim() + '\n```'} />
    </div>
  )
}
