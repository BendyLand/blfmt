# Sorts the file alphabetically
file_path = "src/file-extensions.txt"
lines = []

with open(file_path, "r") as file:
    lines = file.readlines()

lines.sort()

with open(file_path, "w") as file:
    file.writelines(lines)
