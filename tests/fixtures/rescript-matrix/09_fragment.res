@react.component
let make = (~label) => {
  <>
    <h1> {React.string(label)} </h1>
    <p> {React.string("Fragment example")} </p>
  </>
}
