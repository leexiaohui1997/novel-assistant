import { WithAiAction } from '@/components/WithAiAction'
import { useEditorForm } from '@/providers/EditorFormContext'

export function ChapterContentAction() {
  const { novel, chapter, title, content, applyContent } = useEditorForm()

  return (
    <WithAiAction
      tip="AI 生成正文"
      showFeedback
      triggerSize="large"
      triggerButtonProps={{
        variant: 'outlined',
        color: 'default',
      }}
      aiAction={{
        actionName: 'generate_chapter_content',
        getParams: () => ({
          novel_id: novel.id,
          chapter_id: chapter?.id,
          title: title || undefined,
          content: content || undefined,
        }),
      }}
      onResult={(result) => {
        const generatedContent = (result as { content?: string })?.content
        if (generatedContent) {
          applyContent(generatedContent.trim().replace(/\n+/g, '\n'))
        }
      }}
    />
  )
}
