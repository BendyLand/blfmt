# icfmt

A minimally customizable polyglot text-formatter written in Rust!

**icfmt is currently in early development and is non-functional.**

*This document will be updated regularly as the project continues, but the tool is in a purely theoretical state at the moment.*

### About

`icfmt` stands for *Innovative Curiosity Formatter*.

It is a text file formatter which accepts a file extension and directory path as arguments, then applies the configured formatting to each of the matching files in the provided directory. Nice and simple, at least in theory. 

It is also the first piece of software that I am officially releasing under the name *Innovative Curiosity*. Cute.

The idea for this project originated from the desire to have a gofmt style text formatter that worked with other programming languages, as well as other file types, such as plain text files. 

However, unlike gofmt, I would like `icfmt` to be very slightly customizable. I understand that the strictness was an intentional design choice of gofmt, but I personally wanted to allow small changes to be made to the configuration for each file type, such as the amount of characters you want to have on a line before inserting a newline, or how many empty lines you want between blocks (whether you're writing paragraphs or code blocks).
