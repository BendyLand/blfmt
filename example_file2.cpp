#include <iostream>
#include "logging.hpp"
#include "utils.hpp"

using namespace std;

vector<string> extractInnerVariables(string text)
{
    vector<string> result;
    boost::regex pat("\\$\\{(.*?)\\}");

    if (boost::regex_search(text, pat, boost::match_default))
    {
        boost::sregex_iterator iter(text.begin(), text.end(), pat);
        boost::sregex_iterator end;
        while (iter != end) {
            boost::smatch m = *iter;
            for (size_t i = 0; i < m.size(); i++) {
                string item = m[1].str();
                if (strContains(item, "${")) continue;
                result.push_back(item);
            }
            iter++;
        }
    }
    return result;
}

bool containsInnerVariables(string text)
{
    boost::regex pat("\\$\\{(.*)\\}");
    return boost::regex_search(text, pat, boost::match_extra);
}

string removeFirstToken(string line)
{
    size_t i = line.find_first_of(" ");
    return line.substr(i+1);
}

bool containsMultipleArgs(string text)
{
    size_t dQuotes = 0;
    for (size_t i = 0; i < text.size(); i++) {
        if (text[i] == '\"') {
            dQuotes++;
            continue;
        }
        if (dQuotes >= 2 && dQuotes % 2 == 0 && text[i] == ',') return true;
    }
    return false;
}

bool executePrint(string text)
{

    if (containsInnerVariables(text)) {
        vector<string> vars = extractInnerVariables(text);
        vars = vecDedup(vars);

        string line = removeFirstToken(text);

        cout << "log contains variable: " << line << endl;
        for (string var : vars) {
            cout << "\tVar: " << var << endl;
        }
    }
    else {
        if (text.starts_with("print")) {
            string argStr = removeFirstToken(text);
            if (containsMultipleArgs(argStr)) {
                cout << "print multiple args: " << argStr << endl;
            }
            else {
                cout << "print single arg: " <<  argStr << endl;
            }
        }
        else {
            string argStr = removeFirstToken(text);
            if (containsMultipleArgs(argStr)) {
                cout << "puts multiple args: " << argStr << endl;
            }
            else {
                cout << "puts single arg: " <<  argStr << endl;
            }
        }
    }
    return true;
}

/*
* Complications:
* - Variables may be passed before they exist.
*     - Panic.
* - DQuotes may be unmatched.
*     - Panic.
* - Strings may contain escaped characters.
*     - Handle per case.
* - Escaped characters (currently) include: \", \r, \n, \t, \\.
*/
