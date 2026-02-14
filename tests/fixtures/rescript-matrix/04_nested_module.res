module Inner = {
  let helper = () => "inner"
  module Deep = {
    let nested = () => "deep"
  }
}

let access = () => Inner.nested()
