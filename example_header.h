#ifndef STR_H
#define STR_H

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
 * Function to append text to an existing variable of type string*.
 * @param original The base string. 
 * @param suffix The text you want to append onto the original.
 * @returns NOTHING. The text is appended to `original` in place. 
 */ 
void strAppend(string* original, const char* suffix);

/**
 * Creates a copy of the provided string.
 * @param original The string* to copy.
 * @returns An identical string in newly allocated memory.
 */  
string* strCopy(string* original);

/** 
 * Creates a substring out of the given string and the two indices.
 * @param original The string to extract the substring from.
 * @param start The starting index.
 * @param end The ending index (exclusive).
 * @returns A new string* made up of the data extracted `original`
 */  
string* substr(string* original, size_t start, size_t end);

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
 * Destructor function for collections of the string type.
 * @param original The collection of strings to be freed.
 */ 
void strArrFree(string** original);

/** 
 * Function to empty the contents of the string.
 * @param str The string to clear.
 * @returns NOTHING. The contents of `str` are removed in place.
 */ 
void strClear(string* str);

/** 
 * Checks if given string has any data.
 * @param str The string to check.
 * @returns A boolean value representing if the string is empty ("") or not.
 */  
bool strIsEmpty(string* str);

/**
 * Splits a string* into a string** by a given delimiter.
 * @param original The string to split.
 * @param delim The single-char delimiter to split by.
 * @returns A string** containing the parts of the original string.
 * 
 */
string** strSplit(string* original, const char delim);

/**
 * Joins an array of strings (string**) into a string*.
 * @param arr The array of strings to join.
 * @param delim The char* delimiter to join by. 
 * @returns A string* made up of the input array joined by the delimiter. 
 */
string* strArrJoin(string** arr, const char* delim);

/**
 * Finds the length of the provided string array (string**).
 * @param strArr The array to get the length of. 
 * @returns The number of elements in the array.
 */
size_t strArrLen(string** strArr);

#endif
