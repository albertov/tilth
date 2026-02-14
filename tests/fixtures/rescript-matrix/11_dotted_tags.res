@react.component
let make = () => {
  <>
    <React.Fragment>
      <div> {React.string("Nested")} </div>
    </React.Fragment>
    <MySubComponent.SubComponent />
  </>
}

module MySubComponent = {
  @react.component
  let make = () => <div> {React.string("Sub")} </div>

  module SubComponent = {
    @react.component
    let make = () => <span> {React.string("Deep nested")} </span>
  }
}
