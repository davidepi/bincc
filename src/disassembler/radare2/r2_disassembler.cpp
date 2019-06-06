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

        // info
        R2Info info;
        info.from_JSON(r2.exec("ij"));
        exec_arch = info.get_arch();

        // function names
        r2.exec("aaaa");
        json = r2.exec("aflj");

        Json parsed = Json::parse(json);
        char move_to_location_command[2 + 19];
        strcpy(move_to_location_command, "s ");
        int i = 0;
        for(const Json& func_header : parsed)
        {
            i++;
            R2Func function;
            function.from_JSON(func_header.dump());

            // get also the body
            char hex_target[16 + 2 + 1]; // 16 bytes + 0x + \0
            snprintf(hex_target, 19, "0x%X", function.get_offset());
            move_to_location_command[2] = '\0';
            strncat(move_to_location_command, hex_target, 19);
            r2.exec(move_to_location_command);
            std::string json_stmts = r2.exec("pdfj");
            std::vector<std::string> stmts;
            Json body_parsed;
            try
            {
                body_parsed = Json::parse(json_stmts);
            }
            catch(Json::exception& e)
            {
                continue;
                fprintf(stderr, "%s\n", e.what());
            }
            Json stmts_parsed = body_parsed["ops"];
            for(const Json& stmt_parsed : stmts_parsed)
            {
                R2Stmt stmt;
                stmt.from_JSON(stmt_parsed.dump());
                stmts.push_back(stmt.get_opcode());
            }
            function_names.insert(function.get_name());
            function_bodies.insert(
                std::make_pair(function.get_name(), std::move(stmts)));
        }

        r2.close();
    }
}
