export type MarkdownContentProps = {
  content: string
}

export function MarkdownContent({ content }: MarkdownContentProps) {
  return <div>{content}</div>
}
