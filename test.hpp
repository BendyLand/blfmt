#pragma once

#include <string>

namespace my
{
    class string
    {
        std::string m_data;
        string(const std::string& str) : m_data(str) {}
    };
};
