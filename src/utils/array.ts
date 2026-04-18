/**
 * 深度查找数组工具函数
 * 在嵌套数组结构中查找满足条件的元素
 *
 * @param items - 要搜索的数组
 * @param predicate - 判断条件函数，返回 true 表示找到目标
 * @param childrenKey - 子数组的键名，默认为 'children'
 * @returns 找到的第一个匹配元素，未找到返回 null
 *
 * @example
 * // 查找菜单项
 * const menu = deepFindArr(menuItems, (item) => item.path === '/demo1')
 *
 * @example
 * // 自定义子节点键名
 * const result = deepFindArr(treeData, (node) => node.id === 123, 'nodes')
 */
export function deepFindArr<T>(
  items: T[],
  predicate: (item: T) => boolean,
  childrenKey: keyof T = 'children' as keyof T,
): T | null {
  for (const item of items) {
    // 检查当前元素是否满足条件
    if (predicate(item)) {
      return item
    }

    // 递归检查子元素
    const children = item[childrenKey]
    if (Array.isArray(children) && children.length > 0) {
      const found = deepFindArr(children, predicate, childrenKey)
      if (found) {
        return found
      }
    }
  }

  return null
}

/**
 * 深度查找数组并返回路径
 * 在嵌套数组结构中查找满足条件的元素，并返回从根到该元素的路径
 *
 * @param items - 要搜索的数组
 * @param predicate - 判断条件函数
 * @param childrenKey - 子数组的键名，默认为 'children'
 * @returns 找到的元素路径数组，未找到返回空数组
 *
 * @example
 * const path = deepFindArrWithPath(menuItems, (item) => item.key === 'demo1')
 * // 返回: [parentItem, childItem]
 */
export function deepFindArrWithPath<T>(
  items: T[],
  predicate: (item: T) => boolean,
  childrenKey: keyof T = 'children' as keyof T,
): T[] {
  for (const item of items) {
    // 检查当前元素是否满足条件
    if (predicate(item)) {
      return [item]
    }

    // 递归检查子元素
    const children = item[childrenKey]
    if (Array.isArray(children) && children.length > 0) {
      const foundPath = deepFindArrWithPath(children, predicate, childrenKey)
      if (foundPath.length > 0) {
        return [item, ...foundPath]
      }
    }
  }

  return []
}
