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

/*
Large fixture payload for read-mode threshold testing.
These lines intentionally increase token count so this fixture
consistently exercises outline mode under current defaults.

line 001 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 002 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 003 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 004 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 005 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 006 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 007 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 008 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 009 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 010 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 011 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 012 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 013 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 014 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 015 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 016 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 017 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 018 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 019 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 020 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 021 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 022 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 023 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 024 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 025 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 026 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 027 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 028 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 029 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 030 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 031 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 032 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 033 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 034 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 035 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 036 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 037 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 038 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 039 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 040 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 041 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 042 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 043 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 044 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 045 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 046 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 047 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 048 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 049 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 050 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 051 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 052 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 053 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 054 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 055 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 056 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 057 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 058 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 059 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 060 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 061 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 062 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 063 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 064 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 065 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 066 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 067 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 068 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 069 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 070 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 071 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 072 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 073 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 074 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 075 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 076 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 077 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 078 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 079 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 080 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 081 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 082 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 083 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 084 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 085 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 086 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 087 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 088 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 089 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 090 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 091 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 092 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 093 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 094 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 095 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 096 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 097 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 098 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 099 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 100 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 101 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 102 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 103 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 104 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 105 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 106 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 107 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 108 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 109 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 110 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 111 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 112 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 113 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 114 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 115 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 116 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 117 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 118 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 119 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 120 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 121 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 122 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 123 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 124 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 125 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 126 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 127 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 128 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 129 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 130 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 131 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 132 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 133 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 134 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 135 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 136 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 137 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 138 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 139 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 140 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 141 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 142 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 143 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 144 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 145 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 146 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 147 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 148 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 149 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 150 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 151 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 152 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 153 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 154 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 155 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 156 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 157 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 158 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 159 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 160 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 161 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 162 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 163 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 164 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 165 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 166 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 167 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 168 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 169 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 170 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 171 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 172 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 173 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 174 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 175 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 176 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 177 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 178 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 179 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 180 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 181 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 182 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 183 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 184 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 185 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 186 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 187 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 188 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 189 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 190 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 191 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 192 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 193 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 194 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 195 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 196 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 197 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 198 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 199 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
line 200 threshold fixture payload keeps outline mode behavior deterministic for snapshot coverage
*/
