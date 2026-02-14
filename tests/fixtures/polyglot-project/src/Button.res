type color = Red | Green | Blue

let defaultColor = Red

module Theme = {
  let primary = "blue"
  let secondary = "gray"
}

@react.component
let make = (~label: string, ~onClick: unit => unit) => {
  <button onClick={_ => onClick()}> {React.string(label)} </button>
}
