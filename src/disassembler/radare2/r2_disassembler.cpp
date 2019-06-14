#include "r2_disassembler.hpp"
#include "disassembler/function.hpp"
#include "r2_json_parser.hpp"
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
        exec_arch = R2JsonParser::parse_architecture(r2.exec("ij"));

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
            Function function =
                R2JsonParser::parse_function(func_header.dump());

            // get also the body
            char hex_target[16 + 2 + 1]; // 16 bytes + 0x + \0
            snprintf(hex_target, 19, "0x%X", function.get_offset());
            move_to_location_command[2] = '\0';
            strncat(move_to_location_command, hex_target, 19);
            r2.exec(move_to_location_command);
            std::string json_stmts = r2.exec("pdfj");
            std::vector<Statement> stmts;
            Json body_parsed;
            try
            {
                body_parsed = Json::parse(json_stmts);
            }
            catch(Json::exception& e)
            {
                continue;
            }
            Json stmts_parsed = body_parsed["ops"];
            for(const Json& stmt_parsed : stmts_parsed)
            {
                Statement stmt =
                    R2JsonParser::parse_statement(stmt_parsed.dump());
                stmts.push_back(std::move(stmt));
            }
            function_bodies.insert(
                std::make_pair(function.get_name(), std::move(stmts)));
            function_names.insert(std::move(function));
        }
        r2.close();
    }
}
