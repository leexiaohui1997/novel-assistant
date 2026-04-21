/**
 * 通用类型定义
 *
 * 包含项目中可复用的通用类型接口
 */

/**
 * 分页响应结果
 *
 * @template T - 数据类型
 *
 * @example
 * ```typescript
 * const result: PaginatedResult<User> = {
 *   data: [user1, user2],
 *   total: 100
 * }
 * ```
 */
export interface PaginatedResult<T> {
  /** 数据列表 */
  data: T[]
  /** 总数量 */
  total: number
}
