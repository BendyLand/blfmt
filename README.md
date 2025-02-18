# blfmt

A minimally customizable, polyglot text-formatter written in Rust!

### About

`blfmt` stands for *Bland Formatter*. While you could see this as a way to describe its lack of fancy features, it is really just an abbreviation of my name: B. Land(rette).

It is a text file formatter which accepts the path to a file and some optional information as arguments, then applies the configured formatting to the matching file, if it exists.  Nice and simple, at least in theory. 

The idea for this project originated from the desire to have a gofmt style text formatter that worked with other programming languages, as well as other file types, such as plain text files. 

However, unlike gofmt, I would like `blfmt` to be very slightly customizable in specific places. I understand that the strictness was an intentional design choice of gofmt, but I personally wanted to allow small changes to be made to the configuration for each file type, such as the amount of characters in a column, or how many empty lines you want between blocks or paragraphs.

## Usage

Start by building the project via cargo:
```bash
cargo build -r
```
If you would like to use the tool from anywhere in the terminal, move the binary to your local binary directory, or add the directory to your system's PATH:
```bash
# For Unix-like systems
sudo mv target/release/blfmt /usr/local/bin
# OR 
echo 'export PATH="$PATH:$(pwd)/target/release"' >> ~/.bashrc
source ~/.bashrc # apply changes immediately
```

#### Go Files:
```bash
blfmt path/to/file.go
```
 - The underlying Go implementation simply runs gofmt on the specified path, 
    so no additional information is required.

#### Txt Files:
```bash
blfmt path/to/file.txt -o 80 1 -t "Any optional" "Titles" "For where" "You want" "The file split"
```
 - -o is short for --opts (meaning "options").
 - The arguments for the options are the columns and spacing. 
 - If no options are given, the defaults are used (-o 80 1, just like above).
 - If no titles (-t) are given, the program will attempt to infer the location of 
    paragraph breaks based on the existing structure of the text.
    - While this means that the formatter *does* support basic "paragraph inference" 
        to a degree, its behavior is not very consistent. 
 - The most reliable method for formatting text files will always be by manually 
    providing the locations where you want the text to be split, in the form of 
    titles (seen above).

If you have a text file with *extremely* clearly defined paragraphs, 
or if you're just doing a very simple reformat, then you may be lucky enough 
to witness the glory of:
```bash
blfmt path/to/file.txt
```
 - In the cases where this works as intended, it basically feels like magic. 
    - However, these cases are exceptionally rare.
 - Most text files will require a list of titles to be provided in order for them
    to be *accurately* formatted. 
    - Believe it or not, this still tends to be less work than formatting the file manually,
        even if it is a bit tedious in its own way.

 > As cool as it would be, this program does *not* utilize actual magic to function.

#### C/C++ files:

**Note:** *Basic C/C++ files are currently supported, but certain constructs may not work properly. Always make a copy of your files before formatting.*

```bash
blfmt path/to/file.c(pp)
```

 - C/C++ files will be formatted using my personal favorite style. 
 - It is easier to show you than to try to explain it:
```c
// Example C/C++ file format
void example()
{
    size_t condition = 1;
    if (condition) {
        printf("Condition!\n");
        // do some things here
    }
    else {
        printf("No condition :(\n");
        // do some things here instead
    }
}

int main(void)
{
    printf("Here is an example:\n");
    example();
    printf("That was an example\n");
}
```
Make sense? Good!
Don't like it? That's actually pretty reasonable; it's not for everyone. 
But you're in luck! There is support for specifying other styles!

At the moment, the only available styles are the one above (known as the Stroustrup 
variant of K&R) and K&R (can be typed KnR or knr as well). The style can be specified 
like so:

```bash
blfmt path/to/file.c -s knr
blfmt path/to/file.c --style stroustrup
```
In the future, Allman style will be supported. If no style is specified, the 
Stroustrup option will be used by default.
