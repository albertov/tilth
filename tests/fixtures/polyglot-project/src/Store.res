type status = Loading | Ready | Error(string)

type item = {
  id: string,
  name: string,
  description: string,
  price: float,
  quantity: int,
  status: status,
}

type cart = {
  items: array<item>,
  total: float,
  discount: option<float>,
  coupon: option<string>,
}

type user = {
  id: string,
  email: string,
  name: string,
  role: Admin | Member | Guest,
}

type state = {
  cart: cart,
  user: option<user>,
  loading: bool,
  error: option<string>,
  notifications: array<string>,
}

type action =
  | AddItem(item)
  | RemoveItem(string)
  | UpdateQuantity(string, int)
  | ApplyCoupon(string)
  | RemoveCoupon
  | SetUser(user)
  | ClearUser
  | SetLoading(bool)
  | SetError(string)
  | ClearError
  | AddNotification(string)
  | ClearNotifications

external localStorage_getItem: string => Nullable.t<string> = "localStorage.getItem"
external localStorage_setItem: (string, string) => unit = "localStorage.setItem"

let emptyCart = {
  items: [],
  total: 0.0,
  discount: None,
  coupon: None,
}

let initialState = {
  cart: emptyCart,
  user: None,
  loading: false,
  error: None,
  notifications: [],
}

let calculateTotal = (items: array<item>): float => {
  items->Array.reduce(0.0, (acc, item) => {
    acc +. item.price *. Int.toFloat(item.quantity)
  })
}

let applyDiscount = (total: float, discount: option<float>): float => {
  switch discount {
  | Some(d) => total *. (1.0 -. d)
  | None => total
  }
}

let validateCoupon = (code: string): option<float> => {
  switch code {
  | "SAVE10" => Some(0.10)
  | "SAVE20" => Some(0.20)
  | "HALF" => Some(0.50)
  | _ => None
  }
}

let reducer = (state: state, action: action): state => {
  switch action {
  | AddItem(item) =>
    let newItems = state.cart.items->Array.concat([item])
    let newTotal = calculateTotal(newItems)
    {
      ...state,
      cart: {
        ...state.cart,
        items: newItems,
        total: applyDiscount(newTotal, state.cart.discount),
      },
    }
  | RemoveItem(id) =>
    let newItems = state.cart.items->Array.filter(item => item.id !== id)
    let newTotal = calculateTotal(newItems)
    {
      ...state,
      cart: {
        ...state.cart,
        items: newItems,
        total: applyDiscount(newTotal, state.cart.discount),
      },
    }
  | UpdateQuantity(id, qty) =>
    let newItems = state.cart.items->Array.map(item =>
      if item.id === id {
        {...item, quantity: qty}
      } else {
        item
      }
    )
    let newTotal = calculateTotal(newItems)
    {
      ...state,
      cart: {
        ...state.cart,
        items: newItems,
        total: applyDiscount(newTotal, state.cart.discount),
      },
    }
  | ApplyCoupon(code) =>
    switch validateCoupon(code) {
    | Some(discount) =>
      let newTotal = applyDiscount(calculateTotal(state.cart.items), Some(discount))
      {
        ...state,
        cart: {
          ...state.cart,
          total: newTotal,
          discount: Some(discount),
          coupon: Some(code),
        },
      }
    | None => {
        ...state,
        error: Some("Invalid coupon code: " ++ code),
      }
    }
  | RemoveCoupon =>
    let newTotal = calculateTotal(state.cart.items)
    {
      ...state,
      cart: {
        ...state.cart,
        total: newTotal,
        discount: None,
        coupon: None,
      },
    }
  | SetUser(user) => {...state, user: Some(user)}
  | ClearUser => {...state, user: None}
  | SetLoading(loading) => {...state, loading}
  | SetError(msg) => {...state, error: Some(msg)}
  | ClearError => {...state, error: None}
  | AddNotification(msg) => {
      ...state,
      notifications: state.notifications->Array.concat([msg]),
    }
  | ClearNotifications => {...state, notifications: []}
  }
}

module Actions = {
  let addToCart = (dispatch, item: item) => {
    dispatch(AddItem(item))
    dispatch(AddNotification("Added " ++ item.name ++ " to cart"))
  }

  let removeFromCart = (dispatch, id: string) => {
    dispatch(RemoveItem(id))
  }

  let checkout = (dispatch, state: state) => {
    dispatch(SetLoading(true))
    dispatch(ClearError)
    let total = state.cart.total
    if total > 0.0 {
      dispatch(AddNotification("Order placed! Total: $" ++ Float.toString(total)))
      dispatch(SetLoading(false))
    } else {
      dispatch(SetError("Cart is empty"))
      dispatch(SetLoading(false))
    }
  }
}

module Selectors = {
  let getCartItemCount = (state: state): int => {
    state.cart.items->Array.reduce(0, (acc, item) => acc + item.quantity)
  }

  let getCartTotal = (state: state): float => {
    state.cart.total
  }

  let isLoggedIn = (state: state): bool => {
    state.user->Option.isSome
  }

  let getUserName = (state: state): string => {
    switch state.user {
    | Some(user) => user.name
    | None => "Guest"
    }
  }

  let getItemById = (state: state, id: string): option<item> => {
    state.cart.items->Array.find(item => item.id === id)
  }
}

module Persistence = {
  let saveState = (state: state): unit => {
    let json = state.cart.total->Float.toString
    localStorage_setItem("cart_total", json)
  }

  let loadSavedTotal = (): option<float> => {
    let value = localStorage_getItem("cart_total")
    switch value->Nullable.toOption {
    | Some(s) => s->Float.fromString
    | None => None
    }
  }
}

@react.component
let make = (~initialUser: option<user>=?) => {
  let (state, dispatch) = React.useReducer(reducer, {
    ...initialState,
    user: initialUser,
  })

  let itemCount = Selectors.getCartItemCount(state)
  let total = Selectors.getCartTotal(state)
  let userName = Selectors.getUserName(state)

  <div className="store">
    <header>
      <h1> {React.string("Store")} </h1>
      <span> {React.string(userName)} </span>
      <span> {React.string("Items: " ++ Int.toString(itemCount))} </span>
      <span> {React.string("Total: $" ++ Float.toString(total))} </span>
    </header>
    {switch state.error {
    | Some(msg) => <div className="error"> {React.string(msg)} </div>
    | None => React.null
    }}
    {state.loading
      ? <div className="loading"> {React.string("Processing...")} </div>
      : React.null}
    <div className="cart">
      {state.cart.items
      ->Array.map(item =>
        <div key={item.id} className="cart-item">
          <span> {React.string(item.name)} </span>
          <span> {React.string(Float.toString(item.price))} </span>
          <span> {React.string(Int.toString(item.quantity))} </span>
          <button onClick={_ => Actions.removeFromCart(dispatch, item.id)}>
            {React.string("Remove")}
          </button>
        </div>
      )
      ->React.array}
    </div>
    <button onClick={_ => Actions.checkout(dispatch, state)}>
      {React.string("Checkout")}
    </button>
  </div>
}
