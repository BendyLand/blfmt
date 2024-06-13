// to practice formatting go files

package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	file := getFileContents()
	filteredFile := filterFileContents(file)
	blocks := splitIntoBlocks(filteredFile)
	formattedBlocks := filterBlocks(blocks)
	var (
		programmingLanguages    []string
		nonProgrammingLanguages []string
	)
	for _, block := range formattedBlocks {
		lines := strings.Split(block, "\n")
		name := lines[1][:len(lines[1])-2]
		if checkProgrammingLanguage(block) {
			programmingLanguages = append(programmingLanguages, name)
		} else {
			nonProgrammingLanguages = append(nonProgrammingLanguages, name)
		}
	}
	fmt.Println("Programming languages:")
	for _, lang := range programmingLanguages {
		fmt.Println(lang)
	}
	fmt.Println("\nNon-programming languages:")
	for _, lang := range nonProgrammingLanguages {
		fmt.Println(lang)
	}
}

func extractLanguageName(block string) string {
	lines := strings.Split(block, "\n")
	fmt.Println(lines[0])
	return lines[0]
}

func checkProgrammingLanguage(block string) bool {
	lines := strings.Split(block, "\n")
	for _, line := range lines {
		words := strings.Split(line, " ")
		if words[0] == "type:" {
			if words[1] == "programming" {
				return true
			}
		}
	}
	return false
}

func filterBlocks(blocks []string) []string {
	var newBlocks []string
	for _, block := range blocks {
		if len(block) > 5 {
			newBlocks = append(newBlocks, formatBlock(block))
		}
	}
	return newBlocks
}

func formatBlock(block string) string {
	lines := strings.Split(block, " ")
	var (
		pair  string
		pairs []string
	)
	if len(lines) > 1 {
		if !strings.Contains(lines[0], ":") {
			lines[1] = fmt.Sprintf("%s %s", lines[0], lines[1])
			lines = lines[1:]
		}
	}
	for _, line := range lines {
		if strings.Contains(line, ":") {
			pairs = append(pairs, pair)
			pair = ""
			pair += line
		} else {
			pair += " " + line
		}
	}
	return strings.Join(pairs, "\n")
}

func splitIntoBlocks(file string) []string {
	lines := strings.Split(file, "\n")
	var (
		block  string
		blocks []string
	)
	for _, line := range lines {
		if len(line) > 0 {
			if []rune(line)[0] != ' ' {
				blocks = append(blocks, block)
				block = ""
				block += line
			} else {
				block += line
			}
		}
	}
	return blocks
}

func filterFileContents(file string) string {
	lines := strings.Split(file, "\n")
	newStr := ""
	for _, line := range lines {
		if len(line) > 0 {
			if []rune(line)[0] != '#' {
				newStr += line + "\n"
			}
		}
	}
	return newStr
}

func getFileContents() string {
	file, err := os.ReadFile("")
	if err != nil {
		fmt.Println("There was a problem reading the file")
		os.Exit(1)
	}
	return string(file)
}
