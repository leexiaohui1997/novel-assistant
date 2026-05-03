/**
 * 分页响应结果
 * @template T - 数据类型
 */
export interface PaginatedResult<T> {
  /** 数据列表 */
  data: T[]
  /** 总数量 */
  total: number
}
