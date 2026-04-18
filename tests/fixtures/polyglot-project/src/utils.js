export function debounce(fn, delay) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
}

export class EventBus {
  constructor() {
    this.listeners = {};
  }
  on(event, callback) {
    this.listeners[event] = callback;
  }
}
