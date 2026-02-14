package server

type Handler struct {
	Name string
}

func NewHandler(name string) *Handler {
	return &Handler{Name: name}
}

func (h *Handler) ServeHTTP() string {
	return h.Name
}
