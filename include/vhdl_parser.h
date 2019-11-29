#pragma once
#include <exception>
#include <string>

#ifdef __cplusplus
extern "C" bool rust_vhdl_as_json(const char *, char **);
#endif

class rust_exception: : public std::runtime_error
{
public:
    myException(std::string const& msg):
        std::runtime_error(msg)
    {}
};

inline std::string vhdl_as_json(const std::string &str)
{
    char *result;
    if(rust_vhdl_as_json(str.c_str(), &result))
    {
        return std::string(result);
    }
    throw rust_exception({result}));
}
