# blfmt

A minimally customizable, polyglot text-formatter written in Rust!

**blfmt is currently in early development and is only functional for .go and .txt files.**

*This document will be updated regularly as the project continues, but the tool is in a minimally functional state at the moment.*

### About

`blfmt` stands for *Bland Formatter*. While you could see this as a way to describe its lack of fancy features, it is really just an abbreviation of my name: B. Land(rette).

It is a text file formatter which accepts a file extension and directory path as arguments, then applies the configured formatting to each of the matching files in the provided directory. Nice and simple, at least in theory. 

The idea for this project originated from the desire to have a gofmt style text formatter that worked with other programming languages, as well as other file types, such as plain text files. 

However, unlike gofmt, I would like `blfmt` to be very slightly customizable in specific places. I understand that the strictness was an intentional design choice of gofmt, but I personally wanted to allow small changes to be made to the configuration for each file type, such as the amount of characters you want to have on a line before inserting a newline, or how many empty lines you want between blocks (whether you're writing paragraphs or code blocks).

## Usage

In its current state, the easiest way to run this program is through cargo:

```bash
# Go files:
cargo run path/to/file.go
```

```bash
# Text files:
cargo run path/to/file.txt -o 80 1 -t "Any optional" "Titles" "For where" "You want" "The file split"

# -o is short for --opts (meaning "options")
# The arguments for the options are the columns and spacing. 
# If no options are given, the defaults are used (-o 80 1, just like above).

# While this formatter *does* support "paragraph inference" based on line length, 
# its behavior is not very consistent. 
# The most reliable method for formatting text files is by providing the locations
# where you want the text to be split, in the form of titles (seen above).

# If you have a text file with *extremely* clearly defined paragraphs, 
# or if you're just doing a very simple reformat, then you may be lucky enough 
# to witness the glory of:
cargo run path/to/file.txt

# In the cases where this works perfectly, it basically feels like magic. 
# However, these cases are exceptionally rare.
# Most text files will require a list of titles to be provided in order for them
# to be accurately formatted. 
# Believe it or not, this is definitely less work than formatting the file manually,
# even if it is still a bit tedious.

# As cool as it would be, this program does *not* utilize actual magic to function.
```
