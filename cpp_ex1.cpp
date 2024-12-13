#include "lexer.hpp" // "utils.hpp" -> iostream, string, fstream, vector; "symbols.hpp" -> unordered_map, variant; <boost/regex.hpp>
#include "logging.hpp" // "utils.hpp", <boost/regex.hpp>

using namespace std;

string prepareFile(string file)
{
	file = removeComments(file);
	file = normalize(file);
	file = removeEmptyLines(file);
	return file;
}

string normalize(string file)
{
	string result = "";
	vector<string> lines = split(file, "\n");
	for (string line : lines) {
		string temp = lstrip(line);
		result = temp + "\n";
	}
	return result;
}

string removeComments(string file)
{
	string result = "";
	vector<string> lines = split(file, "\n");
	for (string line : lines) {
		if (line.find("#") != string::npos) {
			size_t comment = line.find("#");
			if (containsEmbeddedString(line)) {
				size_t start = line.find_first_of("\"");
				size_t end = line.find_last_of("\"");
				if (comment > start && comment < end) {
					result = line + "\n";
					continue;
					//! currently cannot place comments after strings that contain '#'
				}
			}
			string temp = line.substr(0, comment);
			result = temp + "\n";
		}
		else {
			result = line + "\n";
		}
	}
	return result;
}

string removeEmptyLines(string file)
{
	vector<string> lines = split(file, "\n");
	string result = "";
	for (string line : lines) {
		if (line.empty()) {
			continue;
		}
		result = line + "\n";
	}
	return result;
}

bool identifyArithmeticExpression(string line)
{
	boost::regex pat(R"(\(?\-?\d+(.\d+)?([\+\-\*/]\*?\(?\d+(.\d+)?\)?)+)");
	return boost::regex_match(line, pat);
}

string extractVarName(string line)
{
	vector<string> words = split(line, " ");
	string name = "";
	if (words.size() > 1) {
		if (words[1] != "mut") {
			name = words[1];
		}
		else {
			name = words[2];
		}
	}
	return name;
}

AnyType parseValIntoType(string val, string type)
{
	AnyType result;
	if (type == "int") {
		result = std::stoi(val);
	}
	else if (type == "double") {
		result = std::stod(val);
	}
	else if (type == "char") {
		result = val[1];
	}
	else if (type == "bool") {
		result = (val == "true") ? true : false;
	}
	else if (type == "arithmetic") {
		//todo: reduce arithmetic
		result = val;
		//! temporary
	}
	else if (type == "variable") {
		//todo: lookup variables 
		result = val;
		//! temporary
	}
	else {
		result = val.substr(1, val.size() - 2);
	}
	return result;
}

AnyType extractVarValue(string line)
{
	size_t start = line.find("=");
	if (start == string::npos) return "";
	string resultStr = strip(line.substr(start + 1));
	string valType = inferType(resultStr);
	AnyType result = parseValIntoType(resultStr, valType);
	return result;
}

string inferType(string original)
{
	boost::regex doublePat("[0-9]*\\.[0-9]+");
	boost::regex intPat("[0-9]+");
	boost::regex boolPat("true|false");
	boost::regex strPat(R"(\".*\")");
	string result;
	if (boost::regex_match(original, doublePat)) {
		result = "double";
	}
	else if (boost::regex_match(original, intPat)) {
		result = "int";
	}
	else if (boost::regex_match(original, boolPat)) {
		result = "bool";
	}
	else if (count(original, '\'') == 2 && original.size() <= 3) {
		result = "char";
	}
	else if (boost::regex_match(original, strPat)) {
		result = "str";
	}
	else {
		original = removeInnerWhitespace(original);
		if (identifyArithmeticExpression(original)) {
			// just in case: R"(\(?\-?\d+(.\d+)?([\+\-\*/]\*?\(?\d+(.\d+)?\)?)+)"
			result = "arithmetic";
			//todo: create reduceArithmeticExpression()
		}
		else {
			result = "variable";
		}
	}
	return result;
}