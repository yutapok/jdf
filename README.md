# jdf
## Usage
#### `jdf `

- make json flatten

```
$ cat _sample.json
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
     ]
 }

$ cat _sample.json | jq -c >> sample.json
$ cat sample.json | jdf
{
  ".array.[]": [1,2,3],
  ".bool": false,
  ".int": 123345,
  ".object.sample": 1,
  ".objects.[]": [
      {"key1": "value1"},
      {"key2": "value2"},
      {"key3": "value3"}
  ],
  ".string": "ff3fef323ffv"
}

```
#### `jdf -c </PATH/to/file>`
- execute query for flatten-json

```
$ cat jql
.object.sample AS object_sample

$ cat sample.json | jdf -c jql
{ 
  "object_sample":1,
}

```

- jql file can include multi statement and processed fastly
```
.object.sample AS object_sample
.objects.[] AS objects FLAT_MAP .key1 : .key2
.
.
.
```

```
$ cat sample.json | jdf -c jql
{ 
  "object_sample":1,
  "objects_value1": "value2"  
}

```
