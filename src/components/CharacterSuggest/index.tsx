import { MinusOutlined } from '@ant-design/icons'
import { Button, Space, Table, Tooltip } from 'antd'
import React, { useCallback, useImperativeHandle, useMemo, useState } from 'react'

import { ModelSelect } from '../ModelSelect'

import CreateAction from './CreateAction'

import {
  CharacterModal,
  GeneratedCharacter,
} from '@/pages/CreationDetail/pages/character/components/CharacterModal'
import { Character } from '@/types/character'

export type CharacterSuggestHandle = {
  setSuggests: (suggests: string[]) => void
}

export type CharacterSuggestProps = {
  ref?: React.Ref<CharacterSuggestHandle>
  classNames?: {
    root?: string
  }
  novelId: string
  afterCreated?: (character: Character) => void
}

export default function CharacterSuggest({
  classNames = {},
  novelId,
  ref,
  afterCreated,
}: CharacterSuggestProps) {
  const [suggests, setSuggests] = useState<string[]>([])
  const [selectedModel, setSelectedModel] = useState('')
  const [modalOpen, setModalOpen] = useState(false)
  const [aiCharacter, setAiCharacter] = useState<GeneratedCharacter>()
  const [creatingIndex, setCreatingIndex] = useState(-1)

  const open = useCallback(
    (result: GeneratedCharacter) => {
      if (!modalOpen) {
        setAiCharacter(result)
        setModalOpen(true)
      }
    },
    [modalOpen],
  )

  const tableData = useMemo(
    () =>
      suggests.map((suggest, index) => ({
        index,
        order: index + 1,
        suggest,
      })),
    [suggests],
  )

  const handleDelete = (index: number) => {
    setSuggests((suggests) => {
      return suggests.filter((_, i) => i !== index)
    })
  }

  const afterClose = () => {
    setAiCharacter(void 0)
    setCreatingIndex(-1)
  }

  const handleSuccess = useCallback(
    (character: Character) => {
      afterCreated?.(character)
      handleDelete(creatingIndex)
    },
    [afterCreated, creatingIndex],
  )

  useImperativeHandle(ref, () => ({
    setSuggests,
  }))

  if (!tableData.length) {
    return <></>
  }

  return (
    <div className={`${classNames.root}`}>
      <CharacterModal
        open={modalOpen}
        novelId={novelId}
        onClose={() => setModalOpen(false)}
        onSuccess={handleSuccess}
        afterClose={afterClose}
        aiCharacter={aiCharacter}
      />

      <Table
        size="small"
        rowKey="index"
        pagination={false}
        dataSource={tableData}
        columns={[
          {
            dataIndex: 'order',
            title: '序号',
            width: 60,
            align: 'center',
          },
          {
            dataIndex: 'suggest',
            title: () => (
              <div className="flex items-center gap-2">
                <span>建议</span>
                <div className="flex-1">
                  <ModelSelect
                    useDefault
                    className="w-full"
                    size="small"
                    value={selectedModel}
                    onChange={setSelectedModel}
                  />
                </div>
              </div>
            ),
            ellipsis: true,
          },
          {
            dataIndex: 'action',
            title: '操作',
            width: 100,
            align: 'center',
            render: (_, record) => (
              <Space>
                <CreateAction
                  novelId={novelId}
                  modelId={selectedModel}
                  suggest={record.suggest}
                  onResult={open}
                />
                <Tooltip title="移除">
                  <Button
                    size="small"
                    color="danger"
                    variant="text"
                    icon={<MinusOutlined />}
                    onClick={() => handleDelete(record.index)}
                  />
                </Tooltip>
              </Space>
            ),
          },
        ]}
      />
    </div>
  )
}
