exception ValidationError(string)
exception NotFoundError

let validate = input => {
  if (input == "") {
    raise(ValidationError("Empty input"))
  } else {
    Ok(input)
  }
}
