import { DeleteOutlined, EditOutlined } from '@ant-design/icons'
import { Button, Card, Tag } from 'antd'

import { Character, CharacterGenderLabels, CharacterTypeLabels } from '@/types/character'

interface CharacterCardProps {
  character: Character
  order: number
  onEdit: (character: Character) => void
  onDelete: (character: Character) => void
}

export function CharacterCard({ character, order, onEdit, onDelete }: CharacterCardProps) {
  return (
    <Card
      size="small"
      title={
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-xs text-gray-400"># {order}</span>
            <span>{character.name}</span>
          </div>
          <div className="flex items-center gap-1">
            {character.characterType && (
              <Tag color="blue">类型：{CharacterTypeLabels[character.characterType]}</Tag>
            )}
            <Tag>性别：{CharacterGenderLabels[character.gender]}</Tag>
          </div>
        </div>
      }
      actions={[
        <Button
          key="edit"
          size="small"
          variant="text"
          color="primary"
          icon={<EditOutlined />}
          onClick={() => onEdit(character)}
        >
          编辑
        </Button>,
        <Button
          key="delete"
          size="small"
          variant="text"
          color="danger"
          icon={<DeleteOutlined />}
          onClick={() => onDelete(character)}
        >
          删除
        </Button>,
      ]}
    >
      <div className="h-8 flex items-center">
        <div className="line-clamp-2 leading-4 text-sm">
          {character.appearance ||
            character.background ||
            character.personality ||
            character.additionalInfo ||
            '暂无描述'}
        </div>
      </div>
    </Card>
  )
}
