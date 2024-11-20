#include "tokens.hpp"

Name Token::get_name() const
{
    return this->token.first;
}

std::string Token::get_name_str() const
{
    return name_to_str(this->token.first);
}

std::vector<std::string> Token::get_args() const
{
    return this->token.second;
}

Name str_to_name(std::string name)
{
    static const std::unordered_map<std::string, Name> name_map = {
        {"cp", Name::CP}, {"mv", Name::MV}, {"rm", Name::RM}, 
        {"mkdir", Name::MKDIR}, {"check_env", Name::CHECK_ENV}, 
        {"set_env", Name::SET_ENV}, {"unset_env", Name::UNSET_ENV}, 
        {"exec", Name::EXEC}, {"puts", Name::PUTS}, {"prompt", Name::PROMPT}, 
        {"confirm", Name::CONFIRM}, {"if", Name::IF}, {"then", Name::THEN}, 
        {"else", Name::ELSE}, {"end", Name::END}, {"cond", Name::COND}, 
        {"for", Name::FOR}, {"do", Name::DO}, {"label", Name::LABEL}, 
        {"goto", Name::GOTO}, {"download", Name::DOWNLOAD}, 
        {"variable", Name::VARIABLE}, {"upload", Name::UPLOAD}
    };
    auto it = name_map.find(name);
    return it != name_map.end() ? it->second : Name::UNKNOWN;
}

std::string name_to_str(Name name)
{
    std::string result;
    switch (name) {
        case Name::CP:         result = "cp";        break;
        case Name::MV:         result = "mv";        break;
        case Name::RM:         result = "rm";        break;
        case Name::MKDIR:      result = "mkdir";     break;
        case Name::CHECK_ENV:  result = "check_env"; break;
        case Name::SET_ENV:    result = "set_env";   break;
        case Name::UNSET_ENV:  result = "unset_env"; break;
        case Name::EXEC:       result = "exec";      break;
        case Name::PUTS:       result = "puts";      break;
        case Name::PROMPT:     result = "prompt";    break;
        case Name::CONFIRM:    result = "confirm";   break;
        case Name::IF:         result = "if";        break;
        case Name::THEN:       result = "then";      break;
        case Name::ELSE:       result = "else";      break;
        case Name::END:        result = "end";       break;
        case Name::COND:       result = "cond";      break;
        case Name::FOR:        result = "for";       break;
        case Name::DO:         result = "do";        break;
        case Name::LABEL:      result = "label";     break;
        case Name::GOTO:       result = "goto";      break;
        case Name::DOWNLOAD:   result = "download";  break;
        case Name::UPLOAD:     result = "upload";    break;
        case Name::VARIABLE:   result = "variable";  break;
        case Name::UNKNOWN:    result = "unknown";   break;
        default:               result = "ERROR";     break;
    }
    return result;
}