#pragma once

#include <regex>
#include <iostream>

class Matcher 
{
public: 
    Matcher(const std::string text) 
    {
        try { this->pattern = std::regex(text); }
        catch (const std::regex_error& e) {
            std::cout << "Regex error: " << e.what() << std::endl;
            exit(EXIT_FAILURE);
        }
    }
    void find_token_matches(const std::vector<std::string>& tokens);
    void print_token_matches();
private:
    // Fields
    std::regex pattern;
    std::vector<std::string> matches;
    std::vector<std::string> non_matches;
    // Methods
    bool is_match(const std::string& str);
};

std::string get_pattern();