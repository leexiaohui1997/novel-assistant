import { useMemo } from 'react'

import { MarkdownContent } from './MarkdownContent'

export type JsonContentProps = {
  content: string
}

export function JsonContent({ content }: JsonContentProps) {
  const formatContent = useMemo(() => {
    return '```json\n' + content.trim() + '\n```'
  }, [content])

  return <MarkdownContent content={formatContent} />
}
