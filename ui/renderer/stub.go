package renderer

import "wulfaz/internal/tea"

func Render(m tea.Model) string {
	return m.View()
}
