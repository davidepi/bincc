//
// Created by davide on 6/5/19.
//

#include "r2_disassembler.hpp"
#include "r2_func.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

DisassemblerR2::DisassemblerR2(const char* binary) : Disassembler(binary)
{
    health = r2.set_analyzed_file(binary);
    r2.set_executable(RADARE2_PATH);
}

void DisassemblerR2::analyse()
{
    health &= r2.open();
    if(health)
    {
        std::string json;

        //info
        R2Info info;
        info.from_JSON(r2.exec("ij"));
        exec_arch = info.get_arch();

        //function names
        r2.exec("aaaa");
        json = r2.exec("aflj");

        Json parsed = Json::parse(json);
        for(const Json& func_header : parsed)
        {
            R2Func function;
            function.from_JSON(func_header.dump());
            function_names.insert(function.get_name());
        }
        r2.close();
    }
}
