Status: sketching

It's similar to the
[`generate`](https://www.nushell.sh/commands/docs/generate.html#generate-for-generators)
command, but instead of generating a pipeline with no input, it allows you to
process an input pipeline.

It's also similar to the
[`reduce`](https://www.nushell.sh/commands/docs/reduce.html) command, but it
preserves the pipeline, allowing streaming. With `reduce`, you would accumulate
a list in memory.

## Usage

```nushell
let messages = [
    { type: "data", value: 1 }
    { type: "data", value: 2 }
    { type: "data", value: 3 }
    { type: "threshold" }
    { type: "data", value: 4 }
    { type: "data", value: 5 }
]

# output the last message seen before encountering a “threshold” message,
# then output all subsequent messages in real time

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

## Usage

```
Run closure on each element of a list

Usage:
  > stateful filter <initial> <closure>

Flags:
  -h, --help - Display the help message for this command

Parameters:
  initial <any>: The initial state to pass to the closure
  closure <closure(any, any)>:
    The closure receives `|state, value|` and should return a record in the
    shape: { out?: value, state?: new_state }. Both `out` and `state` are
    optional. You can drop rows by omitting the `out` key, and the current
    state is preserved if its key is omitted.

Input/output types:
  ─#─┬────input────┬───output────
   0 │ list-stream │ list-stream
  ───┴─────────────┴─────────────
```

