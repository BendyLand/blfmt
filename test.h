#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

/**
 * Custom string type.
 * @param data The text of the string.
 * @param length The length of the string.
 */
typedef struct
{
    char* data;
    size_t length;
} string;

/** 
 * Custom string array type.
 * @param entries The strings in the array.
 * @param length The length of the array.
 */
typedef struct
{
    string** entries;
    size_t length;
} stringArray;

/** 
 * Constructor function for the string type.
 * @param text The text to convert to a string.
 * @returns A string* made from the input text.
 */  
string* str(const char* text);

/** 
 * Destructor function for the string type.
 * @param str The string* to be freed.
 */ 
void strFree(string* str);

/** 
 * Function to append text to an existing variable of type string*.
 * @param original The base string. 
 * @param suffix The text you want to append onto the original.
 */ 
void strAppend(string* original, const char* suffix);

/** 
 * Function to empty the contents of the string.
 * @param str The string to clear.
 */ 
void strClear(string* str);

/**
 * Creates a copy of the provided string.
 * @param original The string* to copy.
 * @returns An identical string in newly allocated memory.
 */  
string* strCopy(string* original);

/**
 * Splits a string* into a stringArray* by a given delimiter.
 * @param original The string to split.
 * @param delim The single-char delimiter to split by.
 * @returns A stringArray* containing the parts of the original string.
 */
stringArray* strArr(string* original, char* delim);

/** 
 * Destructor function for the stringArray* type.
 * @param arr The collection of strings to be freed.
 * @returns 1 if everything was successfully freed, 0 otherwise. 
 */ 
void strArrFree(stringArray* arr);

/** 
 * Checks if given string has any data.
 * @param str The string to check.
 * @returns A boolean value representing if the string is empty ("") or not.
 */  
bool strIsEmpty(string* str);

/**
 * Joins an array of strings (stringArray*) into a string*.
 * @param arr The array of strings to join.
 * @param delim The char* delimiter to join by. 
 * @returns A string* made up of the input array joined by the delimiter. 
 */
string* strArrJoin(string** arr, char* delim);

/** 
 * Creates a substring out of the given string and the two indices.
 * @param original The string to extract the substring from.
 * @param start The starting index.
 * @param end The ending index (exclusive).
 * @returns A new string* made up of the data extracted `original`
 */  
string* substr(string* original, size_t start, size_t end);

/**
 * Displays the elements in a string array.
 * @param arr The string array to display.
 */
void strArrDisplay(stringArray* arr);

/** 
 * Checks the provided string for the presence of a certain character.
 * @param haystack The string to search within. 
 * @param needle The character to look for.
 * @returns true if the char was found, false if not.
 */ 
bool strContainsChar(string* haystack, char needle);

/** 
 * Checks the provided string for the presence of a given substring.
 * @param haystack The string to search within. 
 * @param needle The substring to look for.
 * @returns true if the substring was found, false if not.
 */
bool strContainsStr(string* haystack, char* needle);