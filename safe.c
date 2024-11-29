#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

#define BUF_SIZE 3000
#define LINES 50
#define LINE_LEN 500

int find(char* line, const char* delim);
void read_file(const char* path, char* contents);
void get_comment_lines(char* contents, int* nums);
void group_comment_lines(int nums[], int groups[][LINES], size_t* sizes, size_t* count) ;
void insert(char dest[], const char src[], int pos);
void get_lines(char* contents, char lines[][LINE_LEN]);
void find_max_in_groups(char lines[][LINE_LEN], int groups[][LINES], size_t sizes[], size_t count, int max_vals[]);
void fix_comment_lines(char* contents, int groups[][LINES], size_t* sizes, size_t count);
void write_file(const char* path, char* contents);
void trim_end(char *str);

int main(int argc, char** argv)
{
    if (argc > 1) {
        char contents[BUF_SIZE] = {0};
        read_file(argv[1], contents);
        int nums[LINES] = {0};
        get_comment_lines(contents, nums);
        int groups[LINES][LINES] = {0}; 
        size_t group_sizes[LINES] = {0}; 
        size_t group_count = 0;          
        group_comment_lines(nums, groups, group_sizes, &group_count);
        fix_comment_lines(contents, groups, group_sizes, group_count);
        trim_end(contents);
        write_file(argv[1], contents);
        printf("'%s' written successfully!\n", argv[1]);
    }
    else {
        printf("Usage: align-comments <filepath>\n");
        exit(EXIT_FAILURE);
    }
    return 0;
}

void read_file(const char* path, char* contents)
{
    FILE* file = fopen(path, "r");
    if (file) {
        size_t total_read = 0;
        while (fgets(contents + total_read, BUF_SIZE - total_read, file)) {
            total_read = strlen(contents);
            if (total_read >= BUF_SIZE - 1) {
                fprintf(stderr, "Buffer overflow: file content exceeds buffer size\n");
                fclose(file);
                exit(EXIT_FAILURE); 
            }
        }
        fclose(file);
    }
}

void write_file(const char* path, char* contents)
{
    FILE* file = fopen(path, "w");
    if (file) {
        fprintf(file, "%s", contents);
        fclose(file);
    }
}

void get_lines(char* contents, char lines[][LINE_LEN])
{
    size_t current = 0;
    char* start = contents;
    char* newline;
    while (current < 50 && start != NULL && *start != '\0') {
        newline = strchr(start, '\n');
        if (newline != NULL) {
            size_t length = newline - start;
            if (length > 499) length = 499;
            strncpy(lines[current], start, length);
            lines[current][length] = '\0';
            start = newline + 1;
        }
        else {
            strncpy(lines[current], start, 499);
            lines[current][499] = '\0';
            start = NULL;
        }
        current++;
    }
}

void get_comment_lines(char* contents, int* nums)
{
    char lines[50][500];
    get_lines(contents, lines);
    size_t nums_idx = 0;
    for (size_t i = 0; i < LINES; i++) {
        if (strstr(lines[i], "--") != NULL) { 
            nums[nums_idx++] = i;
        }
    }
    nums[nums_idx] = -1;
}


void group_comment_lines(int nums[], int groups[][LINES], size_t* sizes, size_t* count)
{
    size_t current= 0;
    size_t temp_idx = 0;
    for (size_t i = 0; i < LINES; i++) {
        groups[current][temp_idx++] = nums[i];
        if (i == LINES-1 || nums[i+1] - nums[i] > 1) {
            sizes[current++] = temp_idx;
            temp_idx = 0;
        }
    }
    *count = current;
}

void insert(char dest[], const char src[], int pos)
{
    int dest_len = strlen(dest);
    int src_len = strlen(src);
    if (pos < 0 || pos > dest_len) {
        return;
    }
    memmove(dest+pos+src_len, dest+pos, dest_len-pos+1);
    memcpy(dest+pos, src, src_len);
}

void fix_comment_lines(char* contents, int groups[][LINES], size_t* sizes, size_t count)
{
    char lines[LINES][LINE_LEN];
    get_lines(contents, lines);
    int max_vals[LINES] = {0};
    find_max_in_groups(lines, groups, sizes, count, max_vals);
    for (size_t group_idx = 0; group_idx < count; group_idx++) {
        for (size_t line_idx = 0; line_idx < sizes[group_idx]; line_idx++) {
            int line_pos = groups[group_idx][line_idx];
            char* line = lines[line_pos];
            int comment_pos = find(line, "--");
            if (comment_pos >= 0) {
                int diff = max_vals[group_idx] - comment_pos;
                if (diff > 0) {
                    char spaces[LINE_LEN] = {0};
                    memset(spaces, ' ', diff);
                    spaces[diff] = '\0';
                    insert(line, spaces, comment_pos);
                }
            }
        }
    }
    contents[0] = '\0'; 
    for (size_t i = 0; i < LINES; i++) {
        if (strlen(lines[i]) > 0) {
            strcat(contents, lines[i]);
            strcat(contents, "\n");
        }
        else {
            strcat(contents, "\n");
        }
    }
}

int find(char* line, const char* delim)
{
    char* res = strstr(line, delim);
    if (res == NULL) return -1;
    return res - line; 
}

void find_max_in_groups(char lines[][LINE_LEN], int groups[][LINES], size_t sizes[], size_t count, int max_vals[])
{
    for (size_t group_index = 0; group_index < count; group_index++) {
        int max = -1;
        for (size_t i = 0; i < sizes[group_index]; i++) {
            int line_index = groups[group_index][i];
            int comment_pos = find(lines[line_index], "--");
            if (comment_pos > max) {
                max = comment_pos;
            }
        }
        max_vals[group_index] = max;
    }
}


void trim_end(char *str) 
{
    if (str == NULL) return;
    size_t len = strlen(str);
    while (len > 0 && (str[len-1] == ' '  || str[len-1] == '\t' ||
                       str[len-1] == '\n' || str[len-1] == '\r' ||
                       str[len-1] == 0x01)) {
        len--;
    }
    str[len] = '\n'; 
    str[len+1] = '\0';
}