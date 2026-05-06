import { FormInstance } from 'antd'

import { WithAiAction } from '@/components/WithAiAction'
import { useEditorForm } from '@/providers/EditorFormContext'

export type PlotActionProps = {
  label?: string
  novelId: string
  chapterId?: string
  formRef?: React.RefObject<FormInstance | null>
}

export function PlotAction({ label = '本章剧情', novelId, chapterId, formRef }: PlotActionProps) {
  const { title, content } = useEditorForm()

  return (
    <WithAiAction
      tip={`AI 编辑${label}`}
      triggerSize="small"
      placement="rightTop"
      showFeedback
      classNames={{ root: 'w-full items-center!', left: '' }}
      aiAction={{
        actionName: 'edit_chapter_plot',
        getParams: () => ({
          novel_id: novelId,
          chapter_id: chapterId ?? undefined,
          title: title || undefined,
          content: content || undefined,
        }),
      }}
      onResult={(result) => {
        const plot = (result as { plot?: string })?.plot
        if (plot) {
          formRef?.current?.setFieldsValue({ plot })
        }
      }}
    >
      <span>{label}</span>
    </WithAiAction>
  )
}
