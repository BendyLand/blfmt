#include "logging.hpp" // iostream, <boost/regex.hpp>
#include "lexer.hpp" // "utils.hpp" -> iostream, fstream, string, vector
#include <unistd.h>
#include "symbols.hpp" // iostream, variant, unordered_map


void mainLoop(std::vector<std::string>&, SymbolTable&);

int main() {
    std::string file = read_file("../../test.pr");
    Variables global_vars;

    SymbolTable symbols(global_vars);
    file = prepare_file(file);
    std::vector<std::string> lines = split(file, "\n");
    mainLoop(lines, symbols);
    // sleep(10);
    return 0;
}

void mainLoop(std::vector<std::string>& lines, SymbolTable& symbols)
{
    // if for some reason these go negative, size_t wrapping breaks the program
    int num_if_scopes = 0;
    int num_elif_scopes = 0;
    int num_else_scopes = 0;
    int num_for_scopes = 0;
    char last_local_scope = 'g';
    for (std::string line : lines) {
        if (line.starts_with("print") || line.starts_with("puts")) {
            execute_print(line, symbols);
        } else if (line.starts_with("let")) {
            //todo: implement method to choose proper table to insert variable into.
            std::string name = extract_var_name(line);
            AnyType value = extract_var_value(line);
            symbols.add_var(name, value);
        } else {
            if (lstrip(line).starts_with("if")) {
                std::cout << "Handle if statement: " << line << std::endl;
                symbols.new_l_vars("if_"+std::to_string(num_if_scopes));
                last_local_scope = 'i';
                num_if_scopes++;
            }
            else if (contains(line, "elif")) {
                std::cout << "Handle elif: " << line << std::endl;
                if (contains(line, "}")) num_if_scopes--;
                symbols.new_l_vars("elif_"+std::to_string(num_elif_scopes));
                last_local_scope = 'l';
                num_elif_scopes++;
            }
            else if (contains(line, "else")) {
                std::cout << "Handle else: " << line << std::endl;
                if (contains(line, "}")) num_elif_scopes--;
                symbols.new_l_vars("else_"+std::to_string(num_else_scopes));
                last_local_scope = 's';
                num_else_scopes++;
            }
            else if (lstrip(line).starts_with("for")) {
                std::cout << "Handle loop: " << line << std::endl;
                symbols.new_l_vars("for_"+std::to_string(num_for_scopes));
                last_local_scope = 'f';
                num_for_scopes++;
            }
            else {
                if (contains(line, "}")) {
                    switch (last_local_scope) {
                        case 'i':
                            symbols.pop_l_vars("if_"+std::to_string(num_if_scopes));
                            num_if_scopes--;
                            break;
                        case 'l':
                            symbols.pop_l_vars("elif_"+std::to_string(num_elif_scopes));
                            num_elif_scopes--;
                            break;
                        case 's':
                            symbols.pop_l_vars("else_"+std::to_string(num_else_scopes));
                            num_else_scopes--;
                            break;
                        case 'f':
                            symbols.pop_l_vars("for_"+std::to_string(num_for_scopes));
                            num_for_scopes--;
                            break;
                    }
                }
            }

        }
    }

}
