#pragma once

#include "utils.hpp"
#include "os.hpp"

using hex = unsigned char;

class File
{
public:
    std::string name;
    std::string path;
    std::vector<hex> contents;
    size_t size;
    File(const std::string& full_path) 
    {
        extract_path_and_name(full_path, this->path, this->name);
        this->contents = read_file_as_hex(full_path, this->size);
    };
};

inline std::ostream& operator<<(std::ostream& os, const File file)
{
    std::string full_path;
#if OS_UNIX_LIKE_DEFINED
    full_path = file.path + "/" + file.name;
#else
    full_path = file.path + "\\" + file.name;
#endif
    std::cout << "\nFile: " << full_path << " (" << file.size << ")" << std::endl;
    std::cout << "Contents:\n" << file.contents << std::endl;

    return os;
}

class Directory
{
public:
    // Fields
    std::string name;
    std::vector<File> files;
    std::vector<Directory> subdirs;

    // Constructors
    Directory(const std::string& n) : name(n) {}
    Directory(const std::string& n, std::vector<File> f) 
        : name(n), files(f) 
    {}
    Directory(const std::string& n, std::vector<File> f, std::vector<Directory> d) 
        : name(n), files(f), subdirs(d) 
    {}
};

Directory walk_dir(const fs::path& dir_path);

inline std::ostream& operator<<(std::ostream& os, const Directory dir)
{
    std::cout << "Directory: " << dir.name << std::endl;
    std::cout << "Files:" << std::endl;
    for (File file : dir.files) {
        std::cout << file << std::endl;
    }
    std::cout << "Subdirectories:" << std::endl;
    for (Directory dir : dir.subdirs) {
        std::cout << dir << std::endl;
    }
    return os;
}

#if defined(_WIN32) || defined(_WIN64)
    #define OS_WINDOWS
#elif defined(__APPLE__) || defined(__MACH__)
    #define OS_MACOS
#elif defined(__linux__)
    #define OS_LINUX
#elif defined(__FreeBSD__)
    #define OS_FREEBSD
#elif defined(__unix__)
    #define OS_UNIX
#else
    #define OS_UNKNOWN
#endif

#if defined(OS_LINUX)
    #include <sys/wait.h>
    #include <string.h>
#endif

#if defined(OS_MACOS) || defined(OS_LINUX) || defined(OS_UNIX) || defined(OS_FREEBSD)
    #define OS_UNIX_LIKE
#endif 

#if defined(OS_WINDOWS)
    #define WIN32_LEAN_AND_MEAN 
    #include <Windows.h>
#endif

#ifndef OS_UNIX_LIKE_DEFINED
    #define OS_UNIX_LIKE_DEFINED defined(OS_UNIX_LIKE)
#endif

#ifndef OS_WINDOWS_DEFINED
    #define OS_WINDOWS_DEFINED defined(OS_WINDOWS)
#endif

class OS
{
public:
    static std::string detect_os();
    static std::pair<int, std::string> run_command(std::string& args);
private: 
    static std::pair<int, std::string> run_command_unix(const std::vector<std::string>& args);
    static std::pair<int, std::string> run_command_windows(const std::string& command);
};
