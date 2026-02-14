// Interface file for type signatures

type color = Red | Green | Blue
type status = Active | Inactive | Pending

type user = {
  name: string,
  email: string,
  age: int,
}

let create: (name: string, email: string, age: int) => user
let validate: user => result<unit, string>
