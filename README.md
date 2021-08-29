# csv2json

## Usage

help

```
$ csv2json -h
csv2json 0.1.0

USAGE:
    csv2json [FLAGS] [OPTIONS] [CSV]...

FLAGS:
    -h, --help         Prints help information
    -n, --no-header    CSV does not have a header line, so output arrays instead of objects
    -V, --version      Prints version information

OPTIONS:
    -d, --delimiter <delimiter>     By default, it is predicted from the extension
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

Change delimiter, no-header.

```
$ echo "1|2|3\n4|5|6" | csv2json -d '|' --no-header
["1","2","3"]
["4","5","6"] 
```

## Install

```
cargo install --git https://github.com/hinohi/csv2json.git
```
