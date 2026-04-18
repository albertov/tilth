@react.component
let make = (~label) => {
  // Malformed JSX - missing closing tag (intentional test case)
  <div> {React.string(label)} 
}
