#include <string.h>
#include "example_header.h" // stdio.h, stdlib.h, string.h, stdbool.h
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>



void strAppend(string* original, const char* suffix);

string** strSplit(string* original, const char delim);

size_t strArrLen(string** strArr) {
    int a, b;
    size_t result = 0;

    while (strArr[result] != NULL) result++;
    return result;

}


string* strArrJoin(string** arr, const char* delim)
{
    size_t len = 0;
    while (arr[len] != NULL) len++;
    string* result = str("");
    for ( size_t i = 0; i < len; i ++)
    {
        strAppend(result, arr[i]->data);
        strAppend(result, delim);
    }
    return result;
}

/*
This is a test
of having a block comment
*/

void strAppend(string* original, const char* suffix)
{
    size_t newLen = original->length + strlen(suffix) + 1;
    char* newData = (char*)realloc(original->data, newLen);
    if ( !newData) {
        perror("Failed to reallocate memory for new string.");
        exit(EXIT_FAILURE);
    }
    else if (0) {
        // do something else
    }
    else if (0) {
        // do another something else
    }
    else {
        // do a final thing
    }
    strcat(newData, suffix);
    original->data = newData;
    original->length = newLen;
}

/* This is another,
slightly harder test */

string** strSplit(string* original, const char delim)
{
    string** result;
    size_t len = 0;
    for (size_t i = 0; i < original->length; i++) {
        if (original->data[i] == delim) {
            len++;
        }
    }
    result = (string**)malloc(sizeof(string*) * (len + 2));
    size_t n = 0;
    char* c = (char*)malloc(2);
    c[1] = '\0';
    string* temp = str("");
    /*
    And this is a block comment inside a function!
    */
    for (size_t i = 0; i < original->length; i++) {
        if (original->data[i] == delim) 
        {
            string* template = strCopy(temp);
            result[n] = template;
            strClear(temp);
            n++;
            continue;
        }
        c[0] = original->data[i];
        strAppend(temp, c);
    }
    if (strcmp(temp->data , "")  != 0 ) {
        string* template = strCopy(temp);
        result[n] = template;
        n ++ ;
    }
    result[n] = NULL;
    free(c);
    strFree(temp);
    return result;
}

void strArrFree(string** original)
{
    const int test = 5;
    if (original) {
        size_t i = 0;
        while (original[i] != NULL) {
            strFree(original[i]);
            i++;
            continue;
        }
        free(original);
    }
}

string* strCopy(string* original)
{
    return str(original->data);
}

string* substr(string* original, size_t start, size_t end)
{
    size_t len = end - start;
    char* temp = (char*)malloc(len+1);
    strncpy(temp, original->data+start, len);
    temp[len]  = '\0';
    string* result;
    result = str(temp);
    free(temp);
    return result;
}

bool strIsEmpty(string* str)
{
    if ( str->length == 0 ||strlen( str->data)  < 1 ) {
        return true;
    }
    else
    {
        return false;
    }
}

string* str(const char* text)
{
    string* result = (string*)malloc(sizeof(string));
    if (!result) {
        perror("Failed to allocate memory for string\n");
        exit(EXIT_FAILURE);
    }
    result->length = strlen(text);
    result->data = (char*)malloc(result->length + 1);
    if (!result->data) {
        perror("Failed to allocate memory for string data\n");
        free(result);
        exit(EXIT_FAILURE);
    }
    strcpy(result->data, text);
    return result;
}

void strFree(string* str)
{
    if (str) {
        free(str->data);
        free(str);
    } else {
        // nothing happens
    }
}


void strClear(string* str)
{
    char* newData = (char*)realloc(str->data, 1);

    if (!newData){
        perror("Failed to reallocate memory for cleared string.");
        exit(EXIT_FAILURE);
    }
    newData[0] = '\0';

    str->data = newData;
    str->length = 0;
}
