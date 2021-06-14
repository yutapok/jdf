# jdf

## jdf-core
### flatten
- input
```
    {
        "int": 123345,
        "string": "ff3fef323ffv",
        "bool": false,
        "array": [1, 2, 3],
        "object": {
          "sample": 1
        },
        "objects": [
          {"key1": "value1"},
          {"key2": "value2"},
          {"key3": "value3"}
        ],
        "objects2": [
          {"key1": "value1", "key2": "value2"},
          {"key3": "value3", "key4": "value4"}
        ]
    }
```

- output
```
{
  ".array.[0]": 1,
  ".array.[1]": 2,
  ".array.[2]": 3,
  ".bool": false,
  ".int": 123345,
  ".object.sample": 1,
  ".objects.[0].key1": "value1",
  ".objects.[1].key2": "value2",
  ".objects.[2].key3": "value3",
  ".objects2.[0].key1": "value1",
  ".objects2.[0].key2": "value2",
  ".objects2.[1].key3": "value3",
  ".objects2.[1].key4": "value4",
  ".string": "ff3fef323ffv"
}
```
