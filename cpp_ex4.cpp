#include "utils.hpp" // "symbols.hpp" -> iostream, unordered_map, variant; string, fstream, vector
//todo: Implement method to add a locally scoped Variables table

SymbolTable::SymbolTable(Variables vars) : m_vars(vars)
{}

Variables SymbolTable::get_vars()
{
    return this->m_vars;
}

void SymbolTable::add_var(std::string name, AnyType value)
{
    this->m_vars.insert_or_assign(name, value);
}

void SymbolTable::display_vars()
{
    std::cout << "\nCurrent Variables:" << std::endl << std::endl;
    size_t i = this->m_vars.size();
    for (auto& var : this->m_vars) {
        std::string varType = get_var_type(var.second);
        std::cout << "Name: " << var.first << std::endl;
        std::cout << "Value: " << var.second;
        std::cout << "Type: " << varType << std::endl << std::endl;
        i--;
    }
}

bool SymbolTable::contains_all(std::vector<std::string> found_vars)
{
    for (std::string var : found_vars)
    {
        if (!contains_key(this->m_vars, var)) return false;
    }
    return true;
}

AnyType SymbolTable::get_val(std::string name)
{
    for (auto pair : this->m_vars) {
        if (pair.first == name) return pair.second;
    }
    return "VAR_NOT_FOUND";
}
