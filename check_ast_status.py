import difflib

c_file = ""
cpp_file = ""
cpp_specifics = [
    "fn handle_abstract_function_declarator(root: Node, src: String) -> String {",
    "fn handle_abstract_reference_declarator(root: Node, src: String) -> String {",
    "fn handle_access_specifier(root: Node, src: String) -> String {",
    "fn handle_alias_declaration(root: Node, src: String) -> String {",
    "fn handle_catch_clause(root: Node, src: String) -> String {",
    "fn handle_class_specifier(root: Node, src: String) -> String {",
    "fn handle_condition_clause(root: Node, src: String) -> String {",
    "fn handle_field_initializer(root: Node, src: String) -> String {",
    "fn handle_field_initializer_list(root: Node, src: String) -> String {",
    "fn handle_for_range_loop(root: Node, src: String) -> String {",
    "fn handle_lambda_capture_specifier(root: Node, src: String) -> String {",
    "fn handle_lambda_expression(root: Node, src: String) -> String {",
    "fn handle_namespace_alias_definition(root: Node, src: String) -> String {",
    "fn handle_namespace_definition(root: Node, src: String) -> String {",
    "fn handle_nested_namespace_specifier(root: Node, src: String) -> String {",
    "fn handle_new_expression(root: Node, src: String) -> String {",
    "fn handle_qualified_identifier(root: Node, src: String) -> String {",
    "fn handle_reference_declarator(root: Node, src: String) -> String {",
    "fn handle_structured_binding_declarator(root: Node, src: String) -> String {",
    "fn handle_subscript_argument_list(root: Node, src: String) -> String {",
    "fn handle_template_argument_list(root: Node, src: String) -> String {",
    "fn handle_template_declaration(root: Node, src: String) -> String {",
    "fn handle_template_function(root: Node, src: String) -> String {",
    "fn handle_template_parameter_list(root: Node, src: String) -> String {",
    "fn handle_template_type(root: Node, src: String) -> String {",
    "fn handle_try_statement(root: Node, src: String) -> String {",
    "fn handle_type_parameter_declaration(root: Node, src: String) -> String {",
    "fn handle_using_declaration(root: Node, src: String) -> String {",
]

with open("src/c_ast.rs") as file:
    for line in file:
        c_file += line


with open("src/cpp_ast.rs") as file:
    for line in file:
        cpp_file += line

c_fns = []
cpp_fns = []

for line in c_file.split("\n"):
    line = line.strip()
    if line.startswith("fn") or line.startswith("pub fn"):
        c_fns.append(line)

for line in cpp_file.split("\n"):
    line = line.strip()
    if line in cpp_specifics:
        continue
    if line.startswith("fn") or line.startswith("pub fn"):
        cpp_fns.append(line)

c_fns.sort()
cpp_fns.sort()


def get_diffs(c, cpp):
    diffs = []
    for line in c:
        if line not in cpp:
            diffs.append(line)
    for line in cpp:
        if line not in c:
            diffs.append(line)
    return diffs


def separate_fns(c, cpp):
    start = False
    temp = ""
    result = []
    result2 = []
    for line in c_file.split("\n"):
        if line in c_fns:
            start = True
        if start:
            temp += line + "\n"
        if len(line) > 0 and line[0] == "}":
            start = False
            result.append(temp)
            temp = ""
    for line in cpp_file.split("\n"):
        if line in cpp_fns:
            start = True
        if start:
            temp += line + "\n"
        if len(line) > 0 and line[0] == "}":
            start = False
            result2.append(temp)
            temp = ""
    return result, result2


diffs = get_diffs(c_fns, cpp_fns)
sep_c_fns, sep_cpp_fns = separate_fns(c_file, cpp_file)
sep_c_fns.sort()
sep_cpp_fns.sort()


def get_differences(a, b):
    a_lines = a.strip().splitlines()
    b_lines = b.strip().splitlines()
    diff = difflib.unified_diff(a_lines, b_lines, lineterm="")
    return "\n".join(diff)


matched_functions = [
    fn
    for fn in c_fns
    if fn in cpp_fns
    and any(fn in f for f in sep_c_fns)
    and any(fn in f for f in sep_cpp_fns)
]

with open("diffs", "w+") as file:
    for fn in matched_functions:
        full_c = next(f for f in sep_c_fns if fn in f)
        full_cpp = next(f for f in sep_cpp_fns if fn in f)
        if full_c != full_cpp:
            file.write(f"Function: {fn}\n")
            file.write(get_differences(full_c, full_cpp) + "\n\n")

