# csv2json

## Usage

help

```
csv2json 0.1.0

USAGE:
    csv2json [FLAGS] [OPTIONS] [CSV]...

FLAGS:
    -a, --array      Dump JSON Array object instead of Key-Value object
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --delimiter <delimiter>     By default, it is predicted from the extension
    -H, --header <mode>             Change emit header mode [possible values: first-file-only, ff, no, always]
        --gen-completion <shell>    Generate tab-completion scripts for your shell [possible values: zsh, bash, fish,
                                    powershell, elvish]

ARGS:
    <CSV>...    CSV file path
```

Input from stdin

```
$ echo "a,b,c\n1,2,3\n4,5,6" | csv2json
{"a":"1","b":"2","c":"3"}
{"a":"4","b":"5","c":"6"}
```

Predict delimiter from file extension

```
$ echo "a,b\n1,2" > a.csv
$ echo "c\td\n10\t20" > b.tsv
$ csv2json a.csv b.tsv
{"a":"1","b":"2"}
{"c":"10","d":"20"}
```

Change delimiter

```
$ echo "1|2|3\n4|5|6" | csv2json -d '|'
{"1":"4","2":"5","3":"6"}
```

Concat multi csv via jq

```
$ echo "a,b\n1,2" > a.csv
$ echo "b,a\n3,4" > b.csv
$ csv2json a.csv b.csv -H ff
{"a":"a","b":"b"}
{"a":"1","b":"2"}
{"b":"3","a":"4"}
$ csv2json a.csv b.csv -H ff | jq -r '[.a, .b] | @csv'
"a","b"
"1","2"
"4","3"
```

## Install

```
cargo install --git https://github.com/hinohi/csv2json.git
```

## Test

use [PICT docker](https://github.com/iceomix/pict-docker)

```
$ cat test.txt | docker run --rm -i ghcr.io/iceomix/pict
```

| impl | input       | array | header |
|:----:|:------------|:------|:-------|
|  ✅  | stdin       | false | none   |
|  ✅  | stdin       | false | no     |
|  ✅  | stdin       | false | ff     |
|  ✅  | stdin       | true  | always |
|  ✅  | single-path | false | none   |
|  ✅  | single-path | false | always |
|      | single-path | true  | no     |
|      | single-path | true  | ff     |
|  ✅  | many-path   | false | always |
|      | many-path   | true  | none   |
|  ✅  | many-path   | true  | ff     |
|      | many-path   | true  | no     |
