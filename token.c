#include "token.h"

Token parseLineToToken(string* line)
{
    Token result;
    stringArray* words = strArr(line, " ");
    if (strContainsStr(words->entries[0], "let")) {
        result = LET;
    }
    else if (strContainsStr(words->entries[0], "print")) {
        result = PRINT;
    }
    else if (strContainsStr(words->entries[0], "puts")) {
        result = PUTS;
    }
    else if (strContainsStr(words->entries[0], "elif")) {
        result = ELIF;
    }
    else if (strContainsStr(words->entries[0], "if")) {
        result = IF;
    }
    else if (strContainsStr(words->entries[0], "else")) {
        result = ELSE;
    }
    else if (strContainsStr(words->entries[0], "for")) {
        result = FOR;
    }
    else if (strContainsStr(words->entries[0], "{")) {
        result = O_BRACE;
    }
    else if (strContainsStr(words->entries[0], "}")) {
    result = C_BRACE;
    }
    else {
    result = NA;
    }
    strArrFree(words);
    return result;
}

string* tokenToStr(Token token)
{
    string* result;
    switch (token) {
    case LET:
        result = str("LET");
        break;
    case PRINT:
        result = str("PRINT");
        break;
    case PUTS:
        result = str("PUTS");
        break;
    case IF:
        result = str("IF");
        break;
    case ELIF:
        result = str("ELIF");
        break;
    case ELSE:
        result = str("ELSE");
        break;
    case FOR:
        result = str("FOR");
        break;
    case C_BRACE:
        result = str("C_BRACE");
        break;
    case O_BRACE:
        result = str("O_BRACE");
        break;
        default:
        result = str("NA");
        break;
    }
    return result;
}

TokenLine* saveTokenLine(Token token, string* line)
{
    TokenLine* result = (TokenLine*)malloc(sizeof(TokenLine));
    result->token = token;
    string* temp = strCopy(line);
    result->line = temp;
    return result;
}

void tokenLineFree(TokenLine* line)
{
    if (line) {
        if (line->line) {
            strFree(line->line);
        }
        free(line);
    }
}
