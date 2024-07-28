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
