@react.component
let make = (~props) => {
  <div {...props}> {React.string("Spread props")} </div>
}

let component2 = (~extraProps) => {
  <div {...extraProps className="extra"}> {React.string("More spread")} </div>
}
