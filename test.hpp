#pragma once

#include <iostream>
#include <vector>

inline std::ostream& operator<<(std::ostream& os, bool boolean)
{
	if (boolean) {
		std::cout << "true";
	}
	else {
		std::cout << "false";
	}
	return os;
}

template <typename T>
inline std::ostream& operator<<(std::ostream& os, std::vector<T> vec)
{
	size_t length = vec.size();
	for (size_t i = 0; i < length; i++) {
		if (i < length - 1) std::cout << vec[i] << ", ";
		else std::cout << vec[i] << std::endl;
	}
	return os;
}

std::vector<std::string> args_to_vec(int argc, char** argv);
void print_usage();