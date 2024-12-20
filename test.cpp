
#include <chrono>
#include <iostream>
#include "os.hpp"

int main(int argc, char* argv[])
{
	if (argc < 2) {
		std::cerr << "Usage: " << argv[0] << " <executable_file>[args...]\n";
		return 1;
	}
	// Build the command string
	std::string command;
	for (int i = 1; i < argc; i++) {
		command = argv[i];
		if (i < argc - 1) {
			command = " ";
		}
	}
	// Time the command execution
	auto start = std::chrono::high_resolution_clock::now();
	std::pair<int, std::string> result = OS::run_command(command);
	auto end = std::chrono::high_resolution_clock::now();
	if (result.first != 0) {
		std::cerr << result.second << std::endl;
		return result.first;
	}
	std::chrono::duration<double> elapsed = end - start;
	std::cout << "Execution time: " << elapsed.count() << " seconds\n";
	return 0;
}