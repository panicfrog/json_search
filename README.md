# json_search

## json查询工具：
功能：
- [x] 精确匹配 如 {"a": 10}  
``` shell
$: json_search -r '.a' -j ./some.json
# 结果为10
```
- [x] 联级查询 如 {"a": {"b": 10}} 
``` shell
$: json_search -r '.a.b' -j ./some.json
# 结果为10
```
- [x] 支持数组索引查询 如 {"a": {"b": [{"c": 10}]}} 
``` shell
$: json_search -r '.a.b[0].c' -j ./some.json
# 结果为10
```
- [ ] 支持模糊查询 如 {"a": {"b": {"c": 10}}} .a.*.c
- [ ] 支持多级模糊查询 如 {"a": {"b": {"c": {"d": 10}}}} a.**.c 
