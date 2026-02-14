@external("Math", "random")
let mathRandom: unit => float = "Math.random"

@send @module("./path/to/module")
external someExternal: (int, int) => int = "add"
