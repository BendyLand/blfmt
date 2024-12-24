#include "gui.hpp"
#include "os.hpp"
#include "utils.hpp"

using std::cout;
using std::endl;
using std::string;

int main(int argc, /* this is a test*/ char** argv)
{
	QApplication app(argc, argv);
	QWidget window;
	init_window_settings(window);
	window.show();
	int res = app.exec();

	// To bundle all of the contents from the project structure into a header file:
	// string name = std::filesystem::current_path().filename().string() + ".zip";
	// zip_contents(name);
	// size_t size = 0;
	// std::vector<hex> contents = read_file_as_hex(name, size);
	// my_xxd(name);

	return res;
}
