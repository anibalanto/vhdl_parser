#pragma once
#include <exception>
#include <string>

#ifdef __cplusplus
extern "C" bool rust_vhdl_as_json(const char *, char **);
#endif

class rust_exception: public std::exception
{
    const char* _what;
public:
    rust_exception(const char* what) : _what{what} { }

    ~rust_exception()
    {
        delete _what;
    }

    virtual const char* what() const throw()
    {
        return _what;
    }

};

inline std::string vhdl_as_json(const std::string &str)
{
    char *result;
    if(rust_vhdl_as_json(str.c_str(), &result))
    {
        return std::string(result);
    }
    throw rust_exception(result);
}
