#!/bin/bash

# Check if the correct number of arguments is provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <file> <word>"
    exit 1
fi

# Assign arguments to variables
file="$1"
word="$2"

# Use grep to filter lines containing the word, then sort and remove duplicates
grep "$word" "$file" | cut -d':' -f1 | uniq > "temp_output.txt"
mv "temp_output.txt" "output.txt"
