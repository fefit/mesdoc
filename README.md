# ntree

A html node tree Query Selector API.

一个 html 节点树查询接口 API，通过实现该接口可以让节点获得操作 html 结构的能力。

## 接口

### `INodeTrait`

| 方法                                                                           | 参数说明                                                                                              |
| :----------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------- |
| `fn tag_name(&self) -> &str;`                                                  | 获取标签名                                                                                            |
| `fn node_type(&self) -> INodeType;`                                            | 获取标签类型，类型为枚举 `INodeType`                                                                  |
| `fn parent<'b>(&self) -> MaybeResult<'b>;`                                     | 获取父元素                                                                                            |
| `fn child_nodes<'b>(&self) -> Result<'b>;`                                     | 获取所有子元素，包含文本节点、注释节点等                                                              |
| `fn get_attribute(&self, name: &str) -> Option<IAttrValue>;`                   | 获取标签属性值，值为`Option` 枚举 `IAttrValue`                                                        |
| `fn set_attribute(&mut self, name: &str, value: Option<&str>);`                | 设置标签属性值                                                                                        |
| `remove_attribute(&mut self, name: &str);`                                     | 删除标签属性值                                                                                        |
| `fn uuid(&self) -> Option<&str>;`                                              | 获取标签唯一标识符，用来判断两个元素是否同一元素                                                      |
| `fn text_content(&self) -> &str;`                                              | 获取标签文本内容                                                                                      |
| `fn set_text(&mut self, content: &str);`                                       | 设置标签文本内容                                                                                      |
| `fn inner_html(&self) -> &str;`                                                | 获取元素 html                                                                                         |
| `fn outer_html(&self) -> &str;`                                                | 获取元素 html，包含元素自身                                                                           |
| `fn set_html(&mut self, content: &str);`                                       | 设置元素 html                                                                                         |
| `fn remove_child(&mut self, node: BoxDynNode);`                                | 删除元子元素                                                                                          |
| `fn insert_adjacent(&mut self, position: &InsertPosition, node: &BoxDynNode);` | 插入节点，其中`InsertPosition` 为枚举类型，可能值为`BeforeStart`,`AfterStart`,`BeforeEnd`, `AfterEnd` |
| `fn cloned<'b>(&self) -> BoxDynNode<'b>;`                                      | 复制元素，通常为该元素的一个新引用                                                                    |
| `fn to_node(self: Box<Self>) -> Box<dyn Any>;`                                 | 将节点由 trait object 转换为可判定的真实类型节点                                                      |
| `fn owner_document(&self) -> MaybeDocResult;`                                  | 获取元素的当前 document 文档                                                                          |

### `IDocumentTrait`

| 方法                                                                   | 参数说明         |
| :--------------------------------------------------------------------- | :--------------- |
| `fn get_element_by_id<'b>(&self, id: &str) -> Option<BoxDynNode<'b>>;` | 通过 id 查找标签 |

以上即为所有需要实现的接口，实现接口后，将获得类 jQuery API 操作 html 文档的能力，可参照其中的一个实现[https://github.com/fefit/visdom](https://github.com/fefit/visdom)，其 README 内有 API 支持的接口方法。

## 问题 & 建议 & Bugs?

如果您何在使用过程中遇到任问题，或者有好的建议，欢迎提供 Issue. [Issue](https://github.com/fefit/ntree/issues)

## License

[MIT License](./LICENSE).
