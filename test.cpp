#include "gui.hpp"
#include "os.hpp"
#include "utils.hpp"

using std::cout;
using std::endl;
using std::string;
//todo: Create the format for the config file that it reads
//todo: also find places in the gui for the corresponding fields from the config.
int main(int argc, char** argv)
{
	// QApplication app(argc, argv);
	// QWidget window;
	// init_window_settings(window);
	// window.show();
	// int res = app.exec();

	// To bundle all of the contents from the project structure into a header file:
	// string name = std::filesystem::current_path().filename().string() + ".zip";
	// zip_contents(name);
	// size_t size = 0;
	// std::vector<hex> contents = read_file_as_hex(name, size);
	// my_xxd(name);

	u_map<std::string, u_map<std::string, std::any> > test = parse_config_toml("../example.toml");
	//todo: wrap this in a function
	for (auto& entry : test) {
		std::cout << entry.first << std::endl;
		for (auto& inner : entry.second) {
			std::cout << "Key: " << inner.first << std::endl;
			if (inner.second.type() == typeid(std::string)) {
				std::cout << "Value: " << std::any_cast<std::string>(inner.second) << std::endl;
			}
			else if (inner.second.type() == typeid(std::vector<std::string>)) {
				auto vec = std::any_cast<std::vector<std::string>>(inner.second);
				for (auto& entry : vec) {
					std::cout << "Value: " << entry << std::endl;
				}
			}
		}
		std::cout << std::endl;
	}

	// return res;
	return 0;
}
