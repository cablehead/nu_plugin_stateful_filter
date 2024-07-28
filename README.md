Status: proof of concept

This is is Nushell plugin to support stateful filtering.

It's similar to the
[`generate`](https://www.nushell.sh/commands/docs/generate.html#generate-for-generators)
command, but instead of generating a pipeline with no input it allows you to
process an input pipeline.

It's also similar to the
[`reduce`](https://www.nushell.sh/commands/docs/reduce.html) command, but it
preserves the pipeline, allowing streaming, while with reduce you would
accumlate a list in memory.


```nushell
let messages = [
    { type: "data", value: 1 }
    { type: "data", value: 2 }
    { type: "data", value: 3 }
    { type: "threshold" }
    { type: "data", value: 4 }
    { type: "data", value: 5 }
]

$messages | each {|x| sleep 1sec; $x } | stateful filter {found: false} { |state, x|
    if $state.found {
        return { out: $x }
    }

    if $x.type == "threshold" {
        return { state: {found: true}, out: ($state | get last?) }
    }

    { state: {found: false, last: $x} }
}
```

Outputs:

```
{ type: "data", value: 3 }
{ type: "data", value: 4 }
{ type: "data", value: 5 }
```
