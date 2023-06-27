package main

import (
	"bytes"
	"fmt"
	"html/template"
	"io/fs"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"time"

	"github.com/yuin/goldmark"
)

type Post struct {
	Date time.Time
	HTML template.HTML
}

type templateData struct {
	Title string
	Posts []Post
}

func main() {
	err := realMain()
	if err != nil {
		fmt.Fprintf(os.Stderr, "md2html: %s\n", err)
		os.Exit(1)
	}
}

func realMain() error {
	dir := os.Args[1]

	b, err := os.ReadFile(filepath.Join(dir, "index.html"))
	if err != nil {
		return err
	}

	tmpl, err := template.New("index").Parse(string(b))
	if err != nil {
		return err
	}

	fw, err := os.Create("html/index.html")
	if err != nil {
		return err
	}
	defer fw.Close()

	var posts []Post

	pat := regexp.MustCompile(`/(\d{4})/(\d{2})-(\d{2})\.md$`)
	filepath.Walk(dir, func(path string, info fs.FileInfo, walkErr error) error {
		matches := pat.FindStringSubmatch(path)
		if matches == nil {
			return nil
		}
		year, _ := strconv.Atoi(matches[1])
		month, _ := strconv.Atoi(matches[2])
		day, _ := strconv.Atoi(matches[3])

		d := time.Date(year, time.Month(month), day, 0, 0, 0, 0, time.UTC)
		post := Post{Date: d}

		buf := &bytes.Buffer{}
		content, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		err = goldmark.Convert(content, buf)
		if err != nil {
			return err
		}
		post.HTML = template.HTML(buf.String())

		posts = append(posts, post)
		return nil
	})

	td := templateData{
		Title: "2023.8-p.info",
		Posts: posts,
	}
	err = tmpl.Execute(fw, &td)
	if err != nil {
		return err
	}

	return nil
}
