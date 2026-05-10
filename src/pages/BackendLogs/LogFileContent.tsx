import { useEffect } from 'react'

import { useLogContent } from './hooks/useLogContent'

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

  return <div>{content}</div>
}
