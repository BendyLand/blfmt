#include "build.hpp"
#include "gen.hpp"
#include "os.hpp"
#include "utils.hpp"

// To compile on WSL
// g++ -std=c++20 -I/usr/include/lua5.3 -L/usr/lib/x86_64-linux-gnu src/main.cpp src/utils.cpp src/build.cpp src/os.cpp src/gen.cpp -llua5.3 -o main

namespace fs = std::filesystem;

int main(int argc, char** argv)
{
	// Check for template generation
	if (argc > 1) {
		size_t err = check_for_gen_arg(argc, argv);
		switch (err) {
		case 0: 
			return 0;
		case 1: 
			std::cout << "Error checking for gen arg." << std::endl;
			return 1;
			// cases 2+ should fall through
		}
	}
	// Create new Lua state for the config file to use
	lua_State* L = luaL_newstate();
	luaL_openlibs(L);
	// Load and run the Lua config file
	//!switch back to src/blmake.lua for testing
	if (luaL_loadfile(L, "src/blmake.lua") != LUA_OK || lua_pcall(L, 0, 0, 0)) {
		std::cerr << "Failed to load config: " << lua_tostring(L, -1) << std::endl;
		return 1;
	}
	// Check for command line args
	if (argc > 1) {
		size_t err = handle_cl_args(argc, argv, L);
		check_error_fatal((int)err, "Error handling command line arguments.");
		return 0;
	}
	// Handle pre-build scripts
	if (check_pre_build(L)) run_pre_build_script(L);
	// Construct and run compilation command
#if OS_UNIX_LIKE_DEFINED
	fs::path path = get_config_value(L, "src_dir") + "/watcher";
	if (fs::is_directory(path)) {
		std::vector<std::string> commands = handle_incremental_command_construction(L);
		if (commands.size() > 0) {
			std::cout << "Running commands:" << std::endl;
			for (std::string command : commands) {
				std::cout << command << std::endl << std::endl;
			}
			for (std::string command : commands) {
				std::pair<int, std::string> err = OS::run_command(command);
				if (check_error_passive(err.first, err.second)) return 1;
			}
			std::string out_dir = get_lua_str(L, "out_dir");
			std::string output = get_lua_str(L, "output");
			if (fs::is_regular_file(fs::path(out_dir) / fs::path(output))) {
				std::cout << "Compiled successfully!\n" << std::endl;
			}
			else {
				std::cerr << "There was a problem linking the project." << std::endl;
				exit(1);
			}
		}
		else {
			std::cout << "No changes detected. Recompilation not necessary." << std::endl;
		}
	}
	else {
		std::string command = handle_command_construction(L);
		std::cout << command << std::endl << std::endl;
		std::pair<int, std::string> err = OS::run_command(command);
		std::string out_dir = get_lua_str(L, "out_dir");
		std::string output = get_lua_str(L, "output");
		if (fs::is_regular_file(fs::path(out_dir) / fs::path(output))) {
			std::cout << "Compiled successfully!\n" << std::endl;
		}
		else {
			std::cerr << "There was a problem compiling the project." << std::endl;
			exit(1);
		}
	}
#else
	size_t err = Premake::handle_template_generation(L);
	check_error_fatal((int)err, "Error handling command line arguments.");
#endif
	// Handle post-build scripts
	if (check_post_build(L)) run_post_build_script(L);
	// Close lua state
	lua_close(L);
	return 0;
}
