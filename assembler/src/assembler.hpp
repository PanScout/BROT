#pragma once
#include <__ostream/print.h>
#include <algorithm>
#include <cassert>
#include <cctype>
#include <charconv>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <format>
#include <fstream>
#include <print>
#include <ranges>
#include <span>
#include <stdexcept>
#include <string>
#include <system_error>
#include <tuple>
#include <unordered_map>
#include <utility>
#include <vector>

#include "typedefs.hpp"

using std::ifstream;
using std::ofstream;
using std::pair;
using std::println;
using std::string;
using std::vector;

uint64_t process_imm(std::string imm_string, const TypeLabels imm_type, const size_t curr_line_num)
{
    auto detect_base = [](std::string &s) -> int
    {
        if (s.starts_with("0x") || s.starts_with("0X"))
        {
            s.erase(0, 2);
            return 16;
        }
        if (s.starts_with("0b") || s.starts_with("0B"))
        {
            s.erase(0, 2);
            return 2;
        }
        return 10;
    };

    int base = detect_base(imm_string);

    auto parse_int = [&](auto &out)
    {
        auto [ptr, ec] = std::from_chars(imm_string.data(), imm_string.data() + imm_string.size(), out, base);
        if (ec != std::errc{}) throw std::invalid_argument(std::format("Could not convert immediate at line {}", curr_line_num));
    };

    auto parse_fixed = [&](uint64_t frac_bits) -> uint64_t
    {
        double tmp{};
        auto [ptr, ec] = std::from_chars(imm_string.data(), imm_string.data() + imm_string.size(), tmp);
        if (ec != std::errc{}) throw std::invalid_argument(std::format("Could not convert immediate at line {}", curr_line_num));
        int64_t scaled = static_cast<int64_t>(tmp * static_cast<double>(1ULL << frac_bits));
        scaled = std::clamp(scaled, SIMM_MIN, SIMM_MAX);
        return static_cast<uint64_t>(scaled) & MASK_IMM22;
    };

    const uint8_t LAST_VALID_TYPE = 0xA;
    if (std::to_underlying(imm_type) > LAST_VALID_TYPE)
        throw std::invalid_argument(std::format("Invalid immediate type at line: {}", curr_line_num));

    if (imm_type == TypeLabels::Q32_8) return parse_fixed(8);
    if (imm_type == TypeLabels::Q24_16) return parse_fixed(16);
    if (imm_type == TypeLabels::Q8_32) return parse_fixed(32);

    if (imm_type == TypeLabels::BOOL)
    {
        uint64_t val{};
        parse_int(val);
        return (val > 0 ? 1U : 0U);
    }

    if (imm_type == TypeLabels::SINT || imm_type == TypeLabels::SMOD)
    {
        int64_t val{};
        parse_int(val);
        val = std::clamp(val, SIMM_MIN, SIMM_MAX);
        return static_cast<uint64_t>(val) & MASK_IMM22;
    }

    if (imm_type == TypeLabels::UINT || imm_type == TypeLabels::UMOD)
    {
        uint64_t val{};
        parse_int(val);
        val = std::min(val, UIMM_MAX);
        return val & MASK_IMM22;
    }

    throw std::invalid_argument(std::format("Unhandled immediate type at line {}", curr_line_num));
}
vector<pair<string, size_t>> clean_input_file(const string &ASSEMBLY_INPUT)
{
    constexpr string WHITE_SPACE = " \t\r\n";
    // line
    ifstream inputStream(ASSEMBLY_INPUT);
    if (!inputStream.is_open()) throw std::runtime_error("Could not open input file: " + ASSEMBLY_INPUT);

    string currentLine;
    vector<pair<string, size_t>> input_buffer;

    size_t line_num = 1;
    while (getline(inputStream, currentLine)) input_buffer.emplace_back(currentLine, line_num++);

    constexpr uint8_t MAX_ASCII_VALUE = 127;
    for (const auto &[s, curr_line_num] : input_buffer)
        for (const auto &c : s)
            if (static_cast<int8_t>(c) > MAX_ASCII_VALUE)
                throw std::runtime_error(std::format("Non-ASCII character located at line: {}.\nCharacter is {}, value is {:03d}", curr_line_num, c, c));

    //  Delete All comments and leading/preceding white space
    for (auto &&[s, _] : input_buffer)
    {
        if (size_t idx = s.find(';'); idx != std::string::npos) s.erase(idx);
        if (size_t idx = s.find_last_not_of(WHITE_SPACE); idx != std::string::npos) s.erase(idx + 1);
        if (size_t idx = s.find_first_not_of(WHITE_SPACE); idx != std::string::npos) s.erase(0, idx);
    }

    // Delete all empty or white lines
    std::erase_if(input_buffer,
                  [](const pair<string, size_t> &s) { return s.first.empty() || std::ranges::all_of(s.first, isspace); });

    if (input_buffer.size() < 1) throw std::invalid_argument("No actual code provided");

    return input_buffer;
}

std::pair<string, string> process_args(std::span<char *> args)
{
    auto argList = std::vector<std::string>{args.begin() + 1, args.end()};
    string ASSEMBLY_INPUT;
    string ASSEMBLY_OUTPUT;

    if (argList.size() != 3)
        throw std::invalid_argument("Invalid input.\nBasm only supports arguments of the form "
                                    "INPUT_FILE -o INPUT_FILE");
    for (size_t i = 0; i < argList.size(); i++)
    {
        if (argList.at(i) != "-o") ASSEMBLY_INPUT = argList.at(i);

        if (argList.at(i) == "-o")
        {
            if (i + 1 < argList.size()) ASSEMBLY_OUTPUT = argList.at(i + 1);
            else throw std::invalid_argument("No output file provided!");
            i++;
        }
    }

    assert(ASSEMBLY_INPUT.size() > 0);
    assert(ASSEMBLY_OUTPUT.size() > 0);
    return std::pair<string, string>{ASSEMBLY_INPUT, ASSEMBLY_OUTPUT};
}

uint64_t clean_operands_and_embed_tag_and_opcode(InstructionInfo &instruction_info_struct)
{
    uint64_t instruction_encoding = 0;

    for (auto &operand : instruction_info_struct.operands_list)
    {
        if (operand.front() == 'r' || operand.front() == 'R') operand.erase(0, 1);
        if (operand.front() == 'p' || operand.front() == 'P') operand.erase(0, 1);
    }

    // Tag Field
    instruction_encoding |= (static_cast<uint64_t>((TypeLabels::INSTRUCTION)) << START_TAG_FIELD);
    // Opcode
    instruction_encoding |= (static_cast<uint64_t>(instruction_info_struct.label) << START_OPCODE_FIELD);

    return instruction_encoding;
}

uint64_t assemble_r_type(InstructionInfo instruction_info_struct)
{
    uint64_t instruction_encoding = clean_operands_and_embed_tag_and_opcode(instruction_info_struct);

    // BEFORE Registers encoded at bits 13-17, 18-22, 23-27 in steps of 5
    // NOW Registers encoded at bits 17-21, 22-26, 27-31 in steps of 5
    for (uint64_t shift_amount = R_TYPE_START_REGISTER_FIELD; shift_amount <= R_TYPE_END_REGISTER_FIELD;
         shift_amount += R_TYPE_REGISTER_FIELD_WIDTH)
    {
        uint64_t register_val = 0;
        auto result = std::from_chars(instruction_info_struct.operands_list.back().data(),
                                      instruction_info_struct.operands_list.back().data() +
                                          instruction_info_struct.operands_list.back().size(),
                                      register_val);
        if (result.ec != std::errc{})
            throw std::invalid_argument(std::format("Could not determine register number "
                                                    "for instruction {} at line {}",
                                                    instruction_info_struct.instruction_label_str,
                                                    instruction_info_struct.line_num));
        if (register_val > 22 || register_val < 0)
            throw std::runtime_error(
                std::format("Register value of {} is out of range on line {}", register_val, instruction_info_struct.line_num));
        instruction_encoding |= register_val << shift_amount;
        instruction_info_struct.operands_list.pop_back();
    }
    return instruction_encoding;
}

uint64_t assemble_c_type(InstructionInfo instruction_info_struct)
{
    uint64_t instruction_encoding = clean_operands_and_embed_tag_and_opcode(instruction_info_struct);

    string type_coversion_target = instruction_info_struct.operands_list.back();
    for (auto &&c : type_coversion_target) c = static_cast<char>(std::tolower(c));
    instruction_info_struct.operands_list.pop_back();

    for (uint64_t shift_amount = C_TYPE_START_REGISTER_FIELD; shift_amount <= C_TYPE_END_REGISTER_FIELD;
         shift_amount += C_TYPE_REGISTER_FIELD_WIDTH)
    {
        uint64_t register_val = 0;
        auto result = std::from_chars(instruction_info_struct.operands_list.back().data(),
                                      instruction_info_struct.operands_list.back().data() +
                                          instruction_info_struct.operands_list.back().size(),
                                      register_val);
        if (result.ec != std::errc{})
            throw std::invalid_argument(std::format("Could not determine register number "
                                                    "for instruction {} at line {}",
                                                    instruction_info_struct.instruction_label_str,
                                                    instruction_info_struct.line_num));

        instruction_encoding |= register_val << shift_amount;
        instruction_info_struct.operands_list.pop_back();
    }
    if (!STRING_TO_TYPELABEL.contains(type_coversion_target))
    {
        throw std::invalid_argument(std::format("For convert instruction data type {} is invalid", type_coversion_target));
    }

    auto type_coversion_target_bin = static_cast<uint64_t>(STRING_TO_TYPELABEL.at(type_coversion_target));

    instruction_encoding |= (type_coversion_target_bin << C_TYPE_START_TARGET_TYPE_FIELD);
    return instruction_encoding;
}

uint64_t assemble_i_type(InstructionInfo instruction_info_struct)
{
    uint64_t instruction_encoding = clean_operands_and_embed_tag_and_opcode(instruction_info_struct);

    string immm = instruction_info_struct.operands_list.back();
    instruction_info_struct.operands_list.pop_back();

    for (uint64_t shift_amount = I_TYPE_START_REGISTER_FIELD; shift_amount <= I_TYPE_END_REGISTER_FIELD;
         shift_amount += I_TYPE_REGISTER_FIELD_WIDTH)
    {
        uint64_t register_val = 0;
        auto result = std::from_chars(instruction_info_struct.operands_list.back().data(),
                                      instruction_info_struct.operands_list.back().data() +
                                          instruction_info_struct.operands_list.back().size(),
                                      register_val);
        if (result.ec != std::errc{})
            throw std::invalid_argument(std::format("Could not determine register number "
                                                    "for instruction {} at line {}",
                                                    instruction_info_struct.instruction_label_str,
                                                    instruction_info_struct.line_num));

        instruction_encoding |= register_val << shift_amount;
        instruction_info_struct.operands_list.pop_back();
    }

    size_t colon_idx = immm.find(':');
    if (colon_idx == std::string::npos)
        throw std::invalid_argument(
            std::format("Immediate must be typed annonated at line {}", instruction_info_struct.line_num));
    string immm_string = immm.substr(0, colon_idx);

    TypeLabels imm_type{};
    if (const string imm_type_str = immm.substr(colon_idx + 1); !STRING_TO_TYPELABEL.contains(imm_type_str))
        throw std::invalid_argument(
            std::format("Invalid immediate type: {} at line: {}", imm_type_str, instruction_info_struct.line_num));
    else imm_type = STRING_TO_TYPELABEL.find(imm_type_str)->second; // NOLINT

    for (auto &&c : immm_string) c = static_cast<char>(std::tolower(c));

    auto bin_imm = process_imm(immm_string, imm_type, instruction_info_struct.line_num);
    instruction_encoding |= (bin_imm);

    return instruction_encoding;
}

InstructionInfo tokenize_instruction(std::string instruction, const size_t curr_line_num)
{
    constexpr string WHITE_SPACE = " \t\r\n";

    std::vector<string> operands_list;

    string instruction_type = instruction.substr(0, instruction.find_first_of(WHITE_SPACE));
    for (char &c : instruction_type) c = static_cast<char>(std::toupper(c));

    if (!STRING_TO_INFOSTRUCT.contains(instruction_type))
    {
        throw std::invalid_argument(
            std::format("Syntax Error at Line: {} not a valid instruction type: {}", curr_line_num, instruction_type));
    }

    instruction.erase(0, instruction.find_first_of(WHITE_SPACE));

    while (instruction.contains(','))
    {
        string::iterator end_pos = remove(instruction.begin(), instruction.end(), ' ');
        instruction.erase(end_pos, instruction.end());
        operands_list.push_back(instruction.substr(0, instruction.find_first_of(',')));
        instruction.erase(0, instruction.find_first_of(',') + 1);
    }

    operands_list.push_back(instruction);

    auto it = STRING_TO_INFOSTRUCT.find(instruction_type);
    if (it == STRING_TO_INFOSTRUCT.end())
        throw std::invalid_argument(
            std::format("Incorrect or invalid instruction \"{}\" at line number: {}", instruction_type, curr_line_num));

    InstructionInfo instruction_info_struct = it->second;
    instruction_info_struct.operands_list = operands_list;
    instruction_info_struct.line_num = curr_line_num;

    return instruction_info_struct;
}

uint64_t assemble_s_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}
uint64_t assemble_b_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}
uint64_t assemble_u_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}
uint64_t assemble_j_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}
uint64_t assemble_l_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}
uint64_t assemble_p_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}
uint64_t assemble_q_type(InstructionInfo instruction_info_struct)
{
    return instruction_info_struct.line_num;
}

std::vector<string> assemble(const std::vector<std::pair<string, size_t>> &input_buffer)
{
    std::vector<string> output_buffer{};
    output_buffer.reserve(input_buffer.size());

    for (const auto &[instruction, curr_line_num] : input_buffer)
    {
        InstructionInfo instruction_info_struct = tokenize_instruction(instruction, curr_line_num);
        const string INSTRUCTION_STRING_COMMENT = R"( \\)" + instruction; //TODO Use std::format with the struct to pretty print format the comment

        uint64_t instruction_encoding = 0;
        switch (instruction_info_struct.group)
        {
            case InstructionGroup::R_TYPE:
            {
                instruction_encoding = assemble_r_type(instruction_info_struct);
                break;
            }

            case InstructionGroup::C_TYPE:
            {
                instruction_encoding = assemble_c_type(instruction_info_struct);
                break;
            }

            case InstructionGroup::I_TYPE:
            {
                instruction_encoding = assemble_i_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::S_TYPE:
            {
                instruction_encoding = assemble_s_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::B_TYPE:
            {
                instruction_encoding = assemble_b_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::U_TYPE:
            {
                instruction_encoding = assemble_u_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::J_TYPE:
            {
                instruction_encoding = assemble_j_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::L_TYPE:
            {
                instruction_encoding = assemble_l_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::P_TYPE:
            {
                instruction_encoding = assemble_p_type(instruction_info_struct);
                break;
            }
            case InstructionGroup::Q_TYPE:
            {
                instruction_encoding = assemble_q_type(instruction_info_struct);
                break;
            }
        }
        output_buffer.emplace_back(std::format("{:012X}{:12}", instruction_encoding, INSTRUCTION_STRING_COMMENT));
    }
    return output_buffer;
}

void write_buffer(const std::vector<string> &HEX_VECOTOR, const string &ASSEMBLY_OUTPUT)
{
    ofstream outputStream(ASSEMBLY_OUTPUT);
    if (!outputStream.is_open()) throw std::runtime_error("Could not open output file: " + ASSEMBLY_OUTPUT);
    for (const auto &encoded : HEX_VECOTOR) println(outputStream, "{}", encoded);
    outputStream.close();
}
