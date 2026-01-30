package template

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/sblinch/kdl-go"
)

// LoadUnitsFromDir and LoadItemsFromDir load KDL template files from
// directories and register them with a Registry. Each .kdl file may
// contain multiple unit or item definitions.

// LoadUnitsFromDir loads all .kdl files from dir and registers unit templates.
func LoadUnitsFromDir(dir string, reg *Registry) error {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return fmt.Errorf("reading units directory: %w", err)
	}

	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".kdl") {
			continue
		}

		path := filepath.Join(dir, entry.Name())
		if err := loadUnitsFromFile(path, reg); err != nil {
			return fmt.Errorf("loading %s: %w", entry.Name(), err)
		}
	}

	return nil
}

// LoadItemsFromDir loads all .kdl files from dir and registers item templates.
func LoadItemsFromDir(dir string, reg *Registry) error {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return fmt.Errorf("reading items directory: %w", err)
	}

	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".kdl") {
			continue
		}

		path := filepath.Join(dir, entry.Name())
		if err := loadItemsFromFile(path, reg); err != nil {
			return fmt.Errorf("loading %s: %w", entry.Name(), err)
		}
	}

	return nil
}

// loadUnitsFromFile parses a single KDL file and registers any unit nodes.
func loadUnitsFromFile(path string, reg *Registry) error {
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("reading file: %w", err)
	}

	doc, err := kdl.Parse(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("parsing KDL: %w", err)
	}

	filename := filepath.Base(path)
	for _, node := range doc.Nodes {
		if nodeName(node) != "unit" {
			continue
		}

		unit, err := parseUnit(node, filename)
		if err != nil {
			return err
		}

		reg.RegisterUnit(unit.ID, unit)
	}

	return nil
}

// loadItemsFromFile parses a single KDL file and registers any item nodes.
func loadItemsFromFile(path string, reg *Registry) error {
	data, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("reading file: %w", err)
	}

	doc, err := kdl.Parse(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("parsing KDL: %w", err)
	}

	filename := filepath.Base(path)
	for _, node := range doc.Nodes {
		if nodeName(node) != "item" {
			continue
		}

		item, err := parseItem(node, filename)
		if err != nil {
			return err
		}

		reg.RegisterItem(item.ID, item)
	}

	return nil
}
