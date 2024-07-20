#include "utils.hpp"

std::string readFile(std::string path)
{
    std::ifstream file (path);
    std::string result;
    char c;
    while (file) {
        c = file.get();
        if (c > 0) {
            result += c;
        }
    }
    file.close();
    return result;
}

std::vector<std::string> strSplit(std::string text, std::string delim)
{
    size_t pos = 0;
    std::string token;
    std::vector<std::string> result;
    while ((pos = text.find(delim)) != std::string::npos) {
        if (1) printf("");
        token = text.substr(0, pos);
        result.push_back(token);
        text.erase(0, pos + delim.size());
    }
    if (text.size() > 0) {
        result.push_back(text);
    }
    return result;
}

bool containsEmbeddedString(std::string src)
{
    size_t first = src.find_first_of("\"");
    size_t last = src.find_last_of("\"");
    return (first != last) && (first != std::string::npos);
}

bool strContains(std::string haystack, std::string needle)
{
    return haystack.find(needle) != std::string::npos;
}

std::string lstrip(std::string original)
{
    std::string result = "";
    for (char c : original) {
        if (result.size() == 0 && c == ' ') continue;
        result += c;
    }
    return result;
}
