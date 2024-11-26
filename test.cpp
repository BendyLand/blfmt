#include "matcher.hpp"
#include "utils.hpp"

std::string get_pattern()
{
	std::cout << \
	"Welcome to match mode!\n" << \
	"Please enter the regex pattern you would like to check against:" << std::endl;
	std::string pattern;
	std::getline(std::cin, pattern);
	return pattern;
}

// Public
void Matcher::find_token_matches(const std::vector<std::string>& tokens)
{
	std::vector<std::string> matches;
	std::vector<std::string> non_matches;
	matches.reserve(tokens.size());
	non_matches.reserve(tokens.size());
	for (std::string token : tokens) {
		if (is_match(token)) matches.emplace_back(token);
		else non_matches.emplace_back(token);
	}
	this->matches = matches;
	this->non_matches = non_matches;
}

void Matcher::print_token_matches()
{
    auto print_tokens = [](const std::vector<std::string>& tokens, const std::string& message) {
        if (tokens.empty()) std::cout << message << ":\n" << std::endl; 
		else std::cout << message << ":\n" << tokens << std::endl;
    };
    print_tokens(this->non_matches, "The following tokens did not match");
    print_tokens(this->matches, "The following tokens matched");
}

// Private
bool Matcher::is_match(const std::string& str)
{
	return std::regex_search(str, this->pattern);
}