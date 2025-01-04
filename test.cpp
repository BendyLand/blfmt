#include "gen.hpp"
#include "templates/blmake/full_build_template.h"
#include "templates/blmake/build_template.h"
#include "templates/blmake/simple_build_template.h"
#include "templates/blmake/tiny_build_template.h"
#include "templates/blmake/test_build_template.h"
#include "templates/premake/full_build_premake.h"
#include "templates/premake/build_premake.h"
#include "templates/premake/simple_build_premake.h"
#include "templates/premake/tiny_build_premake.h"
#include "templates/premake/test_build_premake.h"
#include "templates/cmake/full_build_cmake.h"
#include "templates/cmake/simple_build_cmake.h"
#include "templates/cmake/tiny_build_cmake.h"
#include "watcher_config.hpp"

namespace Cmake
{
    std::string get_cmake_contents(lua_State* L)
    {
        std::string kind = get_blmake_config_type(L);
        std::string result = "";
        if (kind == "Full_build") {
            for (size_t i = 0; i < full_build_cmake_txt_len; i++) result += full_build_cmake_txt[i];
        }
        else if (kind == "Build") {
            // Build converts roughly to the full cmake config
            for (size_t i = 0; i < full_build_cmake_txt_len; i++) result += full_build_cmake_txt[i];
        }
        else if (kind == "Simple_build") {
            for (size_t i = 0; i < simple_build_cmake_txt_len; i++) result += simple_build_cmake_txt[i];
        }
        else if (kind == "Tiny_build") {
            for (size_t i = 0; i < tiny_build_cmake_txt_len; i++) result += tiny_build_cmake_txt[i];
        }
        else if (kind == "Test_build") {
            // Test_build converts roughly to the full cmake config
            for (size_t i = 0; i < full_build_cmake_txt_len; i++) result += full_build_cmake_txt[i];
        }
        else {
            for (size_t i = 0; i < full_build_cmake_txt_len; i++) result += full_build_cmake_txt[i];
        }
        return result;
    }

    std::string generate_cmake_build(lua_State* L)
    {
        std::string contents = get_cmake_contents(L);
        contents = populate_cmake_template(L, contents);
        return contents;
    }

    size_t handle_template_generation(lua_State* L)
    {
        std::string contents = generate_cmake_build(L);
        size_t err = write_string_to_file("./CMakeLists.txt", contents);
        if (err == 0) std::cout << "CMakeLists.txt generated successfully!" << std::endl;
        return err;
    }

    std::string populate_cmake_template(lua_State* L, const std::string& contents)
    {
        std::string result = "";
        std::string kind = get_blmake_config_type(L);
        //todo: update the if-else pattern to a map to <string, function pointer>
        // first move the getglobal call inside each branching function.
        // then replace the conditional with a map and call based on key presence.
        if (kind == "Full_build") {
            lua_getglobal(L, "Full_build");
            result = populate_full_build_cmake_template(L, contents);
        }
        else if (kind == "Build") {
            lua_getglobal(L, "Build");
            // Build and Full_build have the same config for cmake
            result = populate_full_build_cmake_template(L, contents);
        }
        else if (kind == "Simple_build") {
            lua_getglobal(L, "Simple_build");
            result = populate_simple_build_cmake_template(L, contents);
        }
        else if (kind == "Tiny_build") {
            lua_getglobal(L, "Tiny_build");
            result = populate_tiny_build_cmake_template(L, contents);
        }
        else if (kind == "Test_build") {
            lua_getglobal(L, "Test_build");
            // Test_build and Full_build have the same config for cmake
            result = populate_full_build_cmake_template(L, contents);
        }
        return result;
    }

    void insert_into_quotes(std::vector<std::string>*& vec, const std::string& field, const std::string& value)
    {
        size_t idx = find(*vec, field);
        size_t dq_idx = (*vec)[idx].find("\"");
        if ((*vec)[idx][dq_idx+1] != '"') return;
        std::string new_line = (*vec)[idx].insert(dq_idx+1, value);
        (*vec)[idx] = new_line;
    }

    void insert_after_leading_paren(std::vector<std::string>*& vec, const std::string& field, const std::string& value)
    {
        size_t idx = find(*vec, field);
        size_t p_idx = (*vec)[idx].find("(");
        if (isalnum((*vec)[idx][p_idx+1])) return;
        std::string new_line = (*vec)[idx].insert(p_idx+1, value);
        (*vec)[idx] = new_line;
    }

    void insert_into_middle_of_parens(std::vector<std::string>*& vec, const std::string& field, const std::string& value)
    {
        size_t idx = search(*vec, field);
        size_t s_idx = (*vec)[idx].find("  ");
        if (s_idx == std::string::npos) return;
        std::string new_line = (*vec)[idx].insert(s_idx+1, value);
        (*vec)[idx] = new_line;
    }

    void insert_before_trailing_paren(std::vector<std::string>*& vec, const std::string& field, const std::string& value)
    {
        size_t idx = find(*vec, field);
        size_t p_idx = (*vec)[idx].find(")");
        if (isalnum((*vec)[idx][p_idx+1])) return;
        std::string new_line = (*vec)[idx].insert(p_idx, value);
        remove_space_after_slash(new_line);
        if (new_line.find("//") != std::string::npos) {
            new_line.erase(new_line.find("//"), 1);
        }
        (*vec)[idx] = new_line;
    }

    void insert_after_slash(std::vector<std::string>*& vec, const std::string& field, const std::string& value)
    {
        size_t idx = search(*vec, field) + 2;
        rtrim((*vec)[idx], " ");
        if (!(*vec)[idx].ends_with("/")) return;
        (*vec)[idx] += value;
    }

    void insert_into_multiline_parens(std::vector<std::string>*& vec, const std::string& field, std::string& value)
    {
        size_t idx = find(*vec, field);
        while (!(*vec)[idx].empty()) idx++;
        add_all_leading_tabs(value);
        (*vec)[idx] = value;
    }

    std::string populate_full_build_cmake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string compilation_flags = "";
        compilation_flags += get_lua_table_with_cmds_as_str(L, "warnings", "-W") + " ";
        compilation_flags += get_lua_table_with_cmds_as_str(L, "preproc_opts", "-D") + " ";
        compilation_flags += get_lua_str(L, "profiling") + " ";
        compilation_flags += get_lua_str(L, "optimization") + " ";
        compilation_flags += get_lua_str(L, "lto") + " ";
        compilation_flags += get_lua_table_as_str(L, "platform_opts") + " ";
        trim(compilation_flags, " ");
        std::string files = join(get_lua_table(L, "files"), "\n");
        std::unordered_map<std::string, std::string> str_keywords = {
            {"CMAKE_CXX_COMPILER", get_lua_str(L, "compiler")},
            {"COMPILATION_FLAGS", compilation_flags},
            {"CMAKE_PREFIX_PATH", get_lua_table_as_str(L, "framework_paths")},
        };
        std::string all_deps = get_lua_table_as_str(L, "dependencies") + " " + get_lua_table_as_str(L, "framework_paths");
        all_deps = replace_all(all_deps, " ", "\n");
        std::unordered_map<std::string, std::string> mline_paren_keywords = {
            {"set(SOURCE_FILES", files},
            {"target_link_libraries(", all_deps},
        };
        std::unordered_map<std::string, std::string> leading_paren_keywords = {
            {"project(", get_lua_str(L, "output")},
            {"add_executable(", get_lua_str(L, "output")},
            {"target_compile_options(", get_lua_str(L, "output")},
            {"target_link_libraries(", get_lua_str(L, "output")},
        };
        std::string tbl = get_lua_table_as_str(L, "lang_exts");
        std::string dialect = tbl.find("-std=c++") != std::string::npos ? to_uppercase(tbl.substr(tbl.find("-std=c++")+8, 10)) : "";
        std::unordered_map<std::string, std::string> trailing_paren_keywords = {
            {"CMAKE_CXX_STANDARD", dialect},
            {"CMAKE_CXX_STANDARD_REQUIRED", dialect.size()>0?"ON":"OFF"},
            {"set(CMAKE_BUILD_TYPE", get_lua_str(L, "build_type")},
            {"include_directories(", get_lua_table_as_str(L, "include_dirs")},
            {"CMAKE_RUNTIME_OUTPUT_DIRECTORY ", get_lua_str(L, "out_dir")},
            {"link_directories(", get_lua_table_as_str(L, "linker_opts")},
        };
        std::string prebuild = "";
        std::string postbuild = "";
        lua_getfield(L, -1, "hooks");
        if (lua_istable(L, -1)) {
            lua_getfield(L, -1, "pre_build");
            if (lua_isstring(L, -1)) {
                prebuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
            lua_getfield(L, -1, "post_build");
            if (lua_isstring(L, -1)) {
                postbuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
        }
        lua_pop(L, 1);
        std::unordered_map<std::string, std::string> trailing_slash_keywords = {
            {"PRE_BUILD", prebuild},
            {"POST_BUILD", postbuild},
        };
        std::unordered_map<std::string, std::string> middle_of_parens_keywords = {
            {"PRE_BUILD", get_lua_str(L, "output")},
            {"POST_BUILD", get_lua_str(L, "output")},
        };
        for (auto& [k, v] : str_keywords) {
            insert_into_quotes(lines, k, v);
        }
        for (auto& [k, v] : mline_paren_keywords) {
            insert_into_multiline_parens(lines, k, v);
        }
        for (auto& [k, v] : leading_paren_keywords) {
            insert_after_leading_paren(lines, k, v);
        }
        for (auto& [k, v] : trailing_paren_keywords) {
            insert_before_trailing_paren(lines, k, v);
        }
        for (auto& [k, v] : trailing_slash_keywords) {
            insert_after_slash(lines, k, v);
        }
        for (auto& [k, v] : middle_of_parens_keywords) {
            insert_into_middle_of_parens(lines, k, v);
        }
        return join(*lines, "\n");
    }

    std::string populate_tiny_build_cmake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string files = join(get_lua_table(L, "files"), "\n");
        std::unordered_map<std::string, std::string> mline_paren_keywords = {
            {"SOURCE_FILES", files},
        };
        std::unordered_map<std::string, std::string> leading_paren_keywords = {
            {"project(", get_lua_str(L, "output")},
            {"add_executable(", get_lua_str(L, "output")},
        };
        for (auto& [k, v] : mline_paren_keywords) {
            insert_into_multiline_parens(lines, k, v);
        }
        for (auto& [k, v] : leading_paren_keywords) {
            insert_after_leading_paren(lines, k, v);
        }
        return join(*lines, "\n");
    }

    std::string populate_simple_build_cmake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string files = join(get_lua_table(L, "files"), "\n");
        std::unordered_map<std::string, std::string> str_keywords = {
            {"CMAKE_CXX_COMPILER", get_lua_str(L, "compiler")},
        };
        std::unordered_map<std::string, std::string> mline_paren_keywords = {
            {"SOURCE_FILES", files},
        };
        std::unordered_map<std::string, std::string> leading_paren_keywords = {
            {"project(", get_lua_str(L, "output")},
            {"add_executable(", get_lua_str(L, "output")},
            {"target_link_libraries(", get_lua_str(L, "output")},
        };
        std::string tbl = get_lua_table_as_str(L, "lang_exts");
        std::string dialect = tbl.find("-std=c++") != std::string::npos ? to_uppercase(tbl.substr(tbl.find("-std=c++")+8, 10)) : "";
        std::unordered_map<std::string, std::string> trailing_paren_keywords = {
            {"CMAKE_CXX_STANDARD", dialect},
            {"CMAKE_CXX_STANDARD_REQUIRED", dialect.size()>0?"ON":"OFF"},
            {"include_directories(", get_lua_table_as_str(L, "include_dirs")},
            {"CMAKE_RUNTIME_OUTPUT_DIRECTORY ", get_lua_str(L, "out_dir")},
            {"target_link_libraries(", get_lua_table_as_str(L, "dependencies")},
            {"link_directories(", get_lua_table_as_str(L, "linker_opts")},
        };
        std::string prebuild = "";
        std::string postbuild = "";
        lua_getfield(L, -1, "hooks");
        if (lua_istable(L, -1)) {
            lua_getfield(L, -1, "pre_build");
            if (lua_isstring(L, -1)) {
                prebuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
            lua_getfield(L, -1, "post_build");
            if (lua_isstring(L, -1)) {
                postbuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
        }
        lua_pop(L, 1);
        std::unordered_map<std::string, std::string> trailing_slash_keywords = {
            {"PRE_BUILD", prebuild},
            {"POST_BUILD", postbuild},
        };
        std::unordered_map<std::string, std::string> middle_of_parens_keywords = {
            {"PRE_BUILD", get_lua_str(L, "output")},
            {"POST_BUILD", get_lua_str(L, "output")},
        };
        for (auto& [k, v] : str_keywords) {
            insert_into_quotes(lines, k, v);
        }
        for (auto& [k, v] : mline_paren_keywords) {
            insert_into_multiline_parens(lines, k, v);
        }
        for (auto& [k, v] : leading_paren_keywords) {
            insert_after_leading_paren(lines, k, v);
        }
        for (auto& [k, v] : trailing_paren_keywords) {
            insert_before_trailing_paren(lines, k, v);
        }
        for (auto& [k, v] : trailing_slash_keywords) {
            insert_after_slash(lines, k, v);
        }
        for (auto& [k, v] : middle_of_parens_keywords) {
            insert_into_middle_of_parens(lines, k, v);
        }
        return join(*lines, "\n");
    }

};

// Helper function to get a string field from the Lua table
std::string get_lua_string(lua_State* L, const char* name)
{
    lua_getfield(L, -1, name);
    std::string result = lua_tostring(L, -1);
    lua_pop(L, 1); // Pop the value off the stack
    return result;
}

// Helper function to get a table of strings from the Lua file
std::vector<std::string> get_lua_string_array(lua_State* L, const char* name)
{
    std::vector<std::string> result;
    lua_getfield(L, -1, name); // Push table onto the stack

    if (lua_istable(L, -1)) {
        lua_pushnil(L); // Start iterating with the first key
        while (lua_next(L, -2)) {
            result.push_back(lua_tostring(L, -1));
            lua_pop(L, 1); // Pop the value, keep the key for the next iteration
        }
    }
    lua_pop(L, 1); // Pop the table from the stack
    return result;
}

size_t check_for_gen_arg(int argc, char** argv)
{
    if (argc > 1) {
        if (argv[1] == std::string("gen")) {
            size_t err = Blmake::handle_template_generation(argc, argv);
            return err;
        }
    }
    return 2;
}

size_t handle_cl_args(int argc, char** argv, lua_State* L)
{
    if (argv[1] == std::string("help")) {
        print_help_menu();
        return 0;
    }
    else if (argv[1] == std::string("align")) {
        //todo: write function below
        // align_config_comments();
        return 0;
    }
    else if (argv[1] == std::string("watch")) {
        if (argc > 2) {
            if (argv[2] == std::string("clean")) {
                clean_prev_json(L);
                return 0;
            }
        }
        size_t err = Watcher::generate_watcher_structure(L);
        return err;
    }
    else if (std::string(argv[1]) == "gen") {
        if (std::string(argv[2]) == "premake") {
            size_t err = Premake::handle_template_generation(L);
            return err;
        }
        else {
            size_t err = Cmake::handle_template_generation(L);
            return err;
        }
    }
    else if (std::string(argv[1]) == "premake") {
        size_t err = Premake::handle_template_generation(L);
        return err;
    }
    else if (std::string(argv[1]) == "cmake") {
        size_t err = Cmake::handle_template_generation(L);
        return err;
    }
    return 1;
}

namespace Premake
{
    std::string generate_premake_build(lua_State* L)
    {
        std::string contents = get_premake_contents(L);
        contents = populate_premake_template(L, contents);
        return contents;
    }

    std::string get_premake_contents(lua_State* L)
    {
        std::string kind = get_blmake_config_type(L);
        std::string result = "";
        if (kind == "Full_build") {
            for (size_t i = 0; i < full_build_premake_txt_len; i++) result += full_build_premake_txt[i];
        }
        else if (kind == "Build") {
            for (size_t i = 0; i < build_premake_txt_len; i++) result += build_premake_txt[i];
        }
        else if (kind == "Simple_build") {
            for (size_t i = 0; i < simple_build_premake_txt_len; i++) result += simple_build_premake_txt[i];
        }
        else if (kind == "Tiny_build") {
            for (size_t i = 0; i < tiny_build_premake_txt_len; i++) result += tiny_build_premake_txt[i];
        }
        else if (kind == "Test_build") {
            for (size_t i = 0; i < test_build_premake_txt_len; i++) result += test_build_premake_txt[i];
        }
        else {
            for (size_t i = 0; i < full_build_premake_txt_len; i++) result += full_build_premake_txt[i];
        }
        return result;
    }

    size_t handle_template_generation(lua_State* L)
    {
        std::string contents = generate_premake_build(L);
        contents = populate_premake_template(L, contents);
        size_t err = write_string_to_file("./premake5.lua", contents);
        if (err == 0) std::cout << "Premake file generated successfully!" << std::endl;
        return err;
    }

    void insert_field_str(std::vector<std::string>*& vec, const std::string& field, std::string& value)
    {
        size_t idx = find(*vec, field);
        size_t dq_idx = (*vec)[idx].find("\"");
        if ((*vec)[idx][dq_idx+1] != '"') return;
        std::string new_line = (*vec)[idx].insert(dq_idx+1, value);
        (*vec)[idx] = new_line;
    }

    void insert_field_tbl(std::vector<std::string>*& vec, const std::string& field, std::string& value)
    {
        size_t idx = find(*vec, field);
        size_t b_idx = (*vec)[idx].find("{}");
        if (b_idx == std::string::npos) return;
        value = add_quotes_and_commas(value);
        std::string new_line = (*vec)[idx].insert(b_idx+1, value + " ");
        b_idx = (*vec)[idx].find("{");
        new_line = new_line.insert(b_idx+1, " ");
        (*vec)[idx] = new_line;
    }

    std::string populate_full_build_premake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string tbl = get_lua_table_as_str(L, "lang_exts");
        std::string dialect = tbl.find("-std=c++") != std::string::npos ? to_uppercase(tbl.substr(tbl.find("-std=c++")+5, 10)) : "";
        std::string language = get_lua_str(L, "compiler");
        if (language == "g++") language = "C++";
        else if (language == "gcc" || language == "clang") language = "C";
        std::string out_dir = get_lua_str(L, "out_dir");
        std::unordered_map<std::string, std::string> str_keywords = {
            {"workspace", get_blmake_config_type(L)} ,
            {"location", out_dir},
            {"project", get_blmake_config_type(L)},
            {"language", language},
            {"targetdir", out_dir},
            {"objdir", out_dir.size()>0?out_dir+"/obj":"./obj"},
            {"cppdialect", dialect},
        };
        std::string prefix = get_lua_str(L, "src_dir");
        std::string options = {
            get_lua_table_with_cmds_as_str(L, "warnings", "-W") + " " +
            get_lua_str(L, "lto") + " " +
            get_lua_str(L, "profiling")
        };
        options = remove_extra_spaces(options);
        if (!prefix.empty()) prefix += "/";
        std::string prebuild = "";
        std::string postbuild = "";
        lua_getfield(L, -1, "hooks");
        if (lua_istable(L, -1)) {
            lua_getfield(L, -1, "pre_build");
            if (lua_isstring(L, -1)) {
                prebuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
            lua_getfield(L, -1, "post_build");
            if (lua_isstring(L, -1)) {
                postbuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
        }
        lua_pop(L, 1);
        std::unordered_map<std::string, std::string> tbl_keywords = {
            {"files", get_lua_table_with_cmds_as_str(L, "files", prefix)} ,
            {"includedirs", get_lua_table_as_str(L, "include_dirs")},
            {"defines", get_lua_table_as_str(L, "preproc_opts")},
            {"buildoptions", options},
            {"libdirs", get_lua_table_as_str(L, "linker_opts")},
            {"links", get_lua_table_as_str(L, "dependencies")},
            {"prebuildcommands", prebuild},
            {"postbuildcommands", postbuild},
        };
        for (auto& [k, v] : str_keywords) {
            insert_field_str(lines, k, v);
        }
        for (auto& [k, v] : tbl_keywords) {
            insert_field_tbl(lines, k, v);
        }
        result = join(*lines, "\n");
        return result;
    }

    std::string populate_build_premake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string tbl = get_lua_table_as_str(L, "lang_exts");
        std::string dialect = tbl.find("-std=c++") != std::string::npos ? to_uppercase(tbl.substr(tbl.find("-std=c++")+5, 10)) : "";
        std::string language = get_lua_str(L, "compiler");
        if (language == "g++") language = "C++";
        else if (language == "gcc" || language == "clang") language = "C";
        std::string out_dir = get_lua_str(L, "out_dir");
        std::unordered_map<std::string, std::string> str_keywords = {
            {"workspace", get_blmake_config_type(L)} ,
            {"location", out_dir},
            {"project", get_blmake_config_type(L)},
            {"language", language},
            {"targetdir", out_dir},
            {"objdir", out_dir.size()>0?out_dir+"/obj":"./obj"},
        };
        std::string prefix = get_lua_str(L, "src_dir");
        if (!prefix.empty()) prefix += "/";
        std::unordered_map<std::string, std::string> tbl_keywords = {
            {"files", get_lua_table_with_cmds_as_str(L, "files", prefix)} ,
            {"includedirs", get_lua_table_as_str(L, "include_dirs")},
            {"defines", get_lua_table_as_str(L, "preproc_opts")},
            {"libdirs", get_lua_table_as_str(L, "linker_opts")},
            {"links", get_lua_table_as_str(L, "dependencies")},
        };
        for (auto& [k, v] : str_keywords) {
            insert_field_str(lines, k, v);
        }
        for (auto& [k, v] : tbl_keywords) {
            insert_field_tbl(lines, k, v);
        }
        result = join(*lines, "\n");
        return result;
    }

    std::string populate_simple_build_premake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string language = get_lua_str(L, "compiler");
        if (language == "g++") language = "C++";
        else if (language == "gcc" || language == "clang") language = "C";
        std::unordered_map<std::string, std::string> str_keywords = {
            {"workspace", get_blmake_config_type(L)} ,
            {"location", get_lua_str(L, "out_dir")},
            {"project", get_blmake_config_type(L)},
            {"language", language},
            {"targetdir", "./"},
            {"objdir", "./obj"},
        };
        std::unordered_map<std::string, std::string> tbl_keywords = {
            {"files", get_lua_table_with_cmds_as_str(L, "files", "")} ,
            {"includedirs", get_lua_table_as_str(L, "include_dirs")},
        };
        for (auto& [k, v] : str_keywords) {
            insert_field_str(lines, k, v);
        }
        for (auto& [k, v] : tbl_keywords) {
            insert_field_tbl(lines, k, v);
        }
        result = join(*lines, "\n");
        return result;
    }

    std::string populate_tiny_build_premake_template(lua_State* L, const std::string& contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string language = get_lua_str(L, "compiler");
        if (language == "g++") language = "C++";
        else if (language == "gcc" || language == "clang") language = "C";
        std::unordered_map<std::string, std::string> str_keywords = {
            {"workspace", get_blmake_config_type(L)} ,
            {"location", "./"},
            {"project", get_blmake_config_type(L)},
            {"language", language},
            {"targetdir", "./"},
            {"objdir", "./obj"},
        };
        std::unordered_map<std::string, std::string> tbl_keywords = {
            {"files", get_lua_table_with_cmds_as_str(L, "files", "")} ,
        };
        for (auto& [k, v] : str_keywords) {
            insert_field_str(lines, k, v);
        }
        for (auto& [k, v] : tbl_keywords) {
            insert_field_tbl(lines, k, v);
        }
        result = join(*lines, "\n");
        return result;
    }

    std::string populate_test_build_premake_template(lua_State* L, const std::string contents)
    {
        std::string result = contents;
        std::vector<std::string> temp_vec = split(result, "\n");
        std::vector<std::string>* lines = &temp_vec;
        std::string tbl = get_lua_table_as_str(L, "lang_exts");
        std::string language = get_lua_str(L, "compiler");
        if (language == "g++") language = "C++";
        else if (language == "gcc" || language == "clang") language = "C";
        std::string out_dir = get_lua_str(L, "out_dir");
        std::unordered_map<std::string, std::string> str_keywords = {
            {"workspace", get_blmake_config_type(L)} ,
            {"location", out_dir},
            {"project", get_blmake_config_type(L)},
            {"language", language},
            {"targetdir", out_dir},
            {"objdir", out_dir.size()>0?out_dir+"/obj":"./obj"},
        };
        std::string prefix = get_lua_str(L, "src_dir");
        std::string options = {
            get_lua_table_with_cmds_as_str(L, "warnings", "-W") + " " +
            get_lua_str(L, "profiling")
        };
        options = remove_extra_spaces(options);
        if (!prefix.empty()) prefix += "/";
        std::string prebuild = "";
        std::string postbuild = "";
        lua_getfield(L, -1, "hooks");
        if (lua_istable(L, -1)) {
            lua_getfield(L, -1, "pre_build");
            if (lua_isstring(L, -1)) {
                prebuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
            lua_getfield(L, -1, "post_build");
            if (lua_isstring(L, -1)) {
                postbuild = lua_tostring(L, -1);
            }
            lua_pop(L, 1);
        }
        lua_pop(L, 1);
        std::unordered_map<std::string, std::string> tbl_keywords = {
            {"files", get_lua_table_with_cmds_as_str(L, "files", prefix)} ,
            {"includedirs", get_lua_table_as_str(L, "include_dirs")},
            {"defines", get_lua_table_as_str(L, "preproc_opts")},
            {"buildoptions", options},
            {"libdirs", get_lua_table_as_str(L, "linker_opts")},
            {"links", get_lua_table_as_str(L, "dependencies")},
            {"prebuildcommands", prebuild},
            {"postbuildcommands", postbuild},
        };
        for (auto& [k, v] : str_keywords) {
            insert_field_str(lines, k, v);
        }
        for (auto& [k, v] : tbl_keywords) {
            insert_field_tbl(lines, k, v);
        }
        result = join(*lines, "\n");
        return result;
    }

    std::string populate_premake_template(lua_State* L, const std::string& contents)
    {
        std::string result = "";
        std::string kind = get_blmake_config_type(L);
        //todo: update the if-else pattern to a map to <string, function pointer>
        // first move the getglobal call inside each branching function.
        // then replace the conditional with a map and call based on key presence.
        if (kind == "Full_build") {
            lua_getglobal(L, "Full_build");
            result = populate_full_build_premake_template(L, contents);
        }
        else if (kind == "Build") {
            lua_getglobal(L, "Build");
            result = populate_build_premake_template(L, contents);
        }
        else if (kind == "Simple_build") {
            lua_getglobal(L, "Simple_build");
            result = populate_simple_build_premake_template(L, contents);
        }
        else if (kind == "Tiny_build") {
            lua_getglobal(L, "Tiny_build");
            result = populate_tiny_build_premake_template(L, contents);
        }
        else if (kind == "Test_build") {
            lua_getglobal(L, "Test_build");
            result = populate_test_build_premake_template(L, contents);
        }
        return result;
    }
};

namespace Blmake
{
    size_t generate_full_build()
    {
        std::string contents = "";
        for (size_t i = 0; i < full_build_template_txt_len; i++) contents += full_build_template_txt[i];
        size_t err = write_string_to_file("./blmake.lua", contents);
        if (err == 0) std::cout << "Config file generated successfully!" << std::endl;
        return err;
    }

    size_t generate_build()
    {
        std::string contents = "";
        for (size_t i = 0; i < build_template_txt_len; i++) contents += build_template_txt[i];
        size_t err = write_string_to_file("./blmake.lua", contents);
        if (err == 0) std::cout << "Config file generated successfully!" << std::endl;
        return err;
    }

    size_t generate_simple_build()
    {
        std::string contents = "";
        for (size_t i = 0; i < simple_build_template_txt_len; i++) contents += simple_build_template_txt[i];
        size_t err = write_string_to_file("./blmake.lua", contents);
        if (err == 0) std::cout << "Config file generated successfully!" << std::endl;
        return err;
    }

    size_t generate_tiny_build()
    {
        std::string contents = "";
        for (size_t i = 0; i < tiny_build_template_txt_len; i++) contents += tiny_build_template_txt[i];
        size_t err = write_string_to_file("./blmake.lua", contents);
        if (err == 0) std::cout << "Config file generated successfully!" << std::endl;
        return err;
    }

    size_t generate_test_build()
    {
        std::string contents = "";
        for (size_t i = 0; i < test_build_template_txt_len; i++) contents += test_build_template_txt[i];
        size_t err = write_string_to_file("./blmake.lua", contents);
        if (err == 0) std::cout << "Config file generated successfully!" << std::endl;
        return err;
    }

    size_t handle_template_generation(int argc, char** argv)
    {
        if (argc > 2) {
            std::string check = to_lower(argv[2]);
            if (check == std::string("full")) {
                std::cout << "Generating Full_build in blmake.lua..." << std::endl;
                size_t err = generate_full_build();
                if (err) exit(1);
            }
            else if (check == std::string("build")) {
                std::cout << "Generating Build in blmake.lua..." << std::endl;
                size_t err = generate_build();
                if (err) exit(1);
            }
            else if (check == std::string("simple")) {
                std::cout << "Generating Simple_build in blmake.lua..." << std::endl;
                size_t err = generate_simple_build();
                if (err) exit(1);
            }
            else if (check == std::string("tiny")) {
                std::cout << "Generating Tiny_build in blmake.lua..." << std::endl;
                size_t err = generate_tiny_build();
                if (err) exit(1);
            }
            else if (check == std::string("test")) {
                std::cout << "Generating Test_build in blmake.lua..." << std::endl;
                size_t err = generate_test_build();
                if (err) exit(1);
            }
            else if (check == std::string("premake") || check == std::string("cmake")) {
                return 2;
            }
            else {
                std::cerr << "Unknown template type." << std::endl;
                return 1;
            }
        }
        else {
            std::cout << "Generating Full_build in blmake.lua..." << std::endl;
            size_t err = generate_full_build();
            if (err) exit(1);
        }
        return 0;
    }
};

namespace Watcher
{
    size_t generate_watcher_structure(lua_State* L)
    {
        std::string watcher_binary = (char*)src_watcher_watcher;
        std::string prev_json = reinterpret_cast<const char*>(src_watcher_prev_json);
        std::string src_path = "";
        std::string config_type = get_blmake_config_type(L);
        lua_getglobal(L, config_type.c_str());
        lua_getfield(L, -1, "src_dir");
        if (lua_isstring(L, -1)) {
            std::string src_dir = lua_tostring(L, -1);
            sanitize(src_dir);
            if (!src_dir.empty()) {
                src_path += src_dir + "/";
            }
        }
        lua_pop(L, 1);
        src_path += "watcher";
        std::filesystem::create_directory(src_path);
    #if OS_UNIX_LIKE_DEFINED
        size_t err1 = write_binary_data_to_file(src_path + "/watcher", src_watcher_watcher, src_watcher_watcher_len);
        std::string permissions_cmd = "chmod +x " + src_path + "/watcher";
        OS::run_command(permissions_cmd);
    #else
        size_t err1 = 0; //! temporary until Windows implementation works
    #endif
        size_t err2 = write_string_to_file(src_path + "/prev.json", prev_json);
        size_t err3 = write_string_to_file(src_path + "/recompile_list.txt", "");
        size_t any_err = err1 | err2 | err3;
        if (any_err == 0) std::cout << "Watcher structure generated successfully!" << std::endl;
        return any_err;
    }
};