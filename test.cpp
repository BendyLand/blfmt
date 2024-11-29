#include <filesystem>
#include <iostream>
#include <vector>

namespace fs = std::filesystem;

void walk_dir(const fs::path& dirPath, std::vector<std::string>& fileNames);

int main()
{
	fs::path path = ".";
	std::vector<std::string> names;
	walk_dir(path, names);
	for (std::string name : names) {
		std::cout << name << std::endl;
	}
	return 0;
}

void walk_dir(const fs::path& dirPath, std::vector<std::string>& fileNames)
{
	if (!fs::exists(dirPath) || !fs::is_directory(dirPath)) {
		std::cerr << "Invalid directory: " << dirPath << "\n";
		return;
	}
	for (const auto& entry : fs::directory_iterator(dirPath)) {
		fileNames.emplace_back(entry.path().string());
		if (fs::is_directory(entry)) {
			// Recursive call for subdirectory
			walk_dir(entry, fileNames);
		}
	}
}