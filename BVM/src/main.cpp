#include "typedefs.hpp"
#include <__ostream/print.h>
#include <algorithm>
#include <array>
#include <cassert>
#include <cctype>
#include <charconv>
#include <cstddef>
#include <cstdint>
#include <format>
#include <fstream>
#include <map>
#include <print>
#include <ranges>
#include <stdexcept>
#include <string>
#include <system_error>
#include <unordered_map>
#include <utility>
#include <vector>

using std::ifstream;
using std::string;
using std::vector;

// const uint64_t MASK_41 = (1ULL << 41) - 1;
// const uint64_t UMAX_41 = (1ULL << 41) - 1;
// const int64_t SMAX_41 = (1ULL << 40) - 1;
const uint8_t INSTRUCTION_TYPE_TAG = 0x0C;
// const uint64_t MASK_35 = (1ULL << 35) - 1;
const uint64_t MASK_41 = (1ULL << 41) - 1;
// const uint64_t MASK_IMM18 = (1ULL << 18) - 1;
const uint64_t UMAX_35BIT = (1ULL << 35) - 1;
const int64_t SMAX_35BIT = (1LL << 34) - 1;
const int64_t SMIN_35BIT = -(1LL << 34);
const uint8_t HIGHEST_TYPE_VAL = 0x0E;

uint64_t extract_bits(const uint64_t &val, int high, int low)
{
    int width = high - low + 1;
    uint64_t mask = (1ULL << width) - 1;
    uint64_t ret = (val >> low) & mask;
    return ret;
}

int64_t process_status_to_sat_or_clamp(int64_t result, ArithemeticFlag flag, uint8_t type_bin)
{
    const string type = UINT_TO_TYPE_STRING.find(type_bin)->second;
    if (type == "sint")
    {
        if (flag == ArithemeticFlag::OVERFLOW_F) { return SMAX_35BIT; }
        else if (flag == ArithemeticFlag::UNDERFLOW_F) { return SMIN_35BIT; }
        else { return result; }
    }

    return result;
}

uint64_t process_status_to_sat_or_clamp(uint64_t result, ArithemeticFlag flag, uint8_t type_bin)
{
    const string type = UINT_TO_TYPE_STRING.find(type_bin)->second;
    if (type == "uint")
    {
        if (flag == ArithemeticFlag::OVERFLOW_F) { return UMAX_35BIT; }
        else { return result; }
    }

    return result;
}

void print_status(BVM &vm)
{
    std::ofstream logfile("log.txt");
    if (!logfile.is_open()) { throw std::invalid_argument("Could not open log file!"); }

    std::println(logfile, "=============================");
    std::println(logfile, "‖ {:1} {:^20} ‖", "PC: ", vm.PC);
    std::println(logfile, "===================================================================================================");
    for (size_t i = 0; i < vm.GPR.size(); i++)
    {
        auto type = static_cast<DataType>(extract_bits(vm.GPR[i], 40, 35));
        if (SIGNED_TYPES.contains(type))
        {
            auto val = static_cast<int64_t>(extract_bits(vm.GPR[i], 34, 0));
            std::println(logfile, "‖ {:1} {:2}: {:>8} {:064b} | {:>11} ‖", "GPR", i,
                         UINT_TO_TYPE_STRING.find(static_cast<uint8_t>(type))->second, vm.GPR.at(i), val);
        }
        else
        {
            uint64_t val = extract_bits(vm.GPR[i], 34, 0);
            std::println(logfile, "‖ {:1} {:2}: {:>8} {:064b} | {:>11} ‖", "GPR", i,
                         UINT_TO_TYPE_STRING.find(static_cast<uint8_t>(type))->second, vm.GPR.at(i), val);
        }
    }
    std::println(logfile,
                 "===================================================================================================\n");
}

vector<std::pair<string, size_t>> load_hex(const string &file_name)
{
    if (!file_name.contains(".hex")) throw std::invalid_argument("Input must be a hex file! (ends in .hex)");

    ifstream bvm_input_stream(file_name);
    vector<std::pair<string, size_t>> buffer;

    string curr_line;
    size_t line_num = 1;

    while (std::getline(bvm_input_stream, curr_line)) { buffer.emplace_back(curr_line, line_num++); }

    for (auto &[line, _] : buffer)
    {
        if (auto it = line.find_first_of('\\'); it != std::string::npos) line.erase(it);
        std::erase(line, ' ');
    }

    std::erase_if(buffer, [](std::pair<string, size_t> p) { return p.first.empty() || std::ranges::all_of(p.first, isspace); });

    return buffer;
}

InstructionInfo decode(std::pair<string, size_t> &instruction)
{
    const auto &[instruction_string, curr_line_num] = instruction;
    InstructionInfo instruction_struct{};

    uint64_t curr_instruction_bin = 0;
    auto from_chars_result = std::from_chars(instruction_string.data(), instruction_string.data() + instruction_string.size(),
                                             curr_instruction_bin, 16);
    if (from_chars_result.ec != std::errc{})
        throw std::invalid_argument(std::format("Could not decode instruction {} at line {}", instruction_string, curr_line_num));

    uint64_t instruction_tag_field = (extract_bits(curr_instruction_bin, END_TAG_FIELD, START_TAG_FIELD));
    if (instruction_tag_field != INSTRUCTION_TYPE_TAG)
        throw std::invalid_argument(std::format("Not an intstruction type at line number: {}", curr_line_num));

    uint64_t opcode_field = (extract_bits(curr_instruction_bin, END_OPCODE_FIELD, START_OPCODE_FIELD));

    auto it = LABEL_TO_INFOSTRUCT.find(static_cast<InstructionLabel>(opcode_field));
    if (it == LABEL_TO_INFOSTRUCT.end())
        throw std::invalid_argument(
            std::format("Invalid opcode: {:B}, not found in lookup table at line {}", opcode_field, curr_line_num));

    instruction_struct = it->second;
    instruction_struct.bin = curr_instruction_bin;
    instruction_struct.line_num = curr_line_num;

    return instruction_struct;
    // NOLINT(readability-else-after-return)
}

void execute_r_type(BVM &vm, InstructionInfo instruction)
{
    switch (instruction.label)
    {

        case InstructionLabel::ADD:
        {
            break;
        }
        case InstructionLabel::SUB:
        {
            break;
        }
        case InstructionLabel::XOR:
        {
            break;
        }
        case InstructionLabel::OR:
        {
            break;
        }
        case InstructionLabel::AND:
        {
            break;
        }
        case InstructionLabel::SLL:
        {
            break;
        }
        case InstructionLabel::SRL:
        {
            break;
        }
        case InstructionLabel::SRA:
        {
            break;
        }
        case InstructionLabel::MUL:
        {
            break;
        }
        case InstructionLabel::MULH:
        {
            break;
        }
        case InstructionLabel::MAC:
        {
            break;
        }
        default:
        {
            // SHOULD NOT BE HERE EVER
            break;
        }
    }
}
void execute_i_type(BVM &vm, const InstructionInfo& instruction)
{
    // Register Numbers
    const uint64_t rd = extract_bits(instruction.bin, 27, 23);
    const uint64_t rs1 = extract_bits(instruction.bin, 22, 18);
    // Immediate/Register Total Values (Including type)
    const uint64_t rs1_bin = vm.GPR.at(rs1);
    const auto type = static_cast<DataType>(extract_bits(rs1_bin, 40, 35));

    if (static_cast<uint8_t>(type) > HIGHEST_TYPE_VAL || type == DataType::Free)
        throw std::invalid_argument(std::format("Not a valid type of operand register source [1] [{:}] at line [{:X}]",
                                                DATATYPE_TO_STRING.find(type)->second, instruction.line_num));

    if (uint64_t type_dest = (extract_bits(vm.GPR.at(rd), END_TAG_FIELD, START_TAG_FIELD)); type_dest > HIGHEST_TYPE_VAL)
        throw std::invalid_argument(
            std::format("Not a valid type of register destimation, is inert at line {:X}", instruction.line_num));

    // Implicitly zero extended
    uint64_t imm = extract_bits(instruction.bin, 17, 0);
    uint64_t rs1_val = extract_bits(rs1_bin, 34, 0);
    uint64_t result = 0;

    if (SIGNED_TYPES.contains(type))
    {
        // Sign Extend Instead
        rs1_val = static_cast<uint64_t>(((static_cast<int64_t>(rs1_val) << 29ULL) >> 29ULL));
        imm = static_cast<uint64_t>((static_cast<int64_t>(imm) << 46ULL) >> 46ULL);
    }

    switch (instruction.label)
    {
        case InstructionLabel::ADDI:
        {
            result = rs1_val + imm;

            // Saturation Check
            if (SIGNED_TYPES.contains(type) && type != DataType::Smod)
            {
                int64_t rs1_val_signed = static_cast<int64_t>(rs1_val);
                int64_t imm_signed = static_cast<int64_t>(imm);
                if ((rs1_val_signed > 0 && imm_signed > 0) && result < 0)
                {
                    // Two Postives become negative, overflow
                    result = SMAX_35BIT;
                }
                else if (((rs1_val_signed < 0) && (imm_signed < 0)) && result > 0)
                {
                    // Two negatives become postive, underflow
                    result = SMIN_35BIT;
                }
            }
            else if (UNSIGNED_TYPES.contains(type) && type != DataType::Umod)
            {
                // Overflow saturate
                if (result > UMAX_35BIT) result = UMAX_35BIT; // NOLINT
            }
            break;
        }
        case InstructionLabel::XORI:
        {
            result = rs1_val ^ imm;
            break;
        }
        case InstructionLabel::ORI:
        {
            result = rs1_val | imm;
            break;
        }
        case InstructionLabel::ANDI:
        {
            result = rs1_val & imm;
            break;
        }
        case InstructionLabel::SLLI: // THIS NEEDS TO BE CHANGED, FOR BROT WE ONLY NEED SHIFT LEFT AND SHIFT RIGHT.
        {
            break;
        }
        case InstructionLabel::SRLI:
        {
            break;
        }
        case InstructionLabel::SRAI:
        {
            break;
        }
        default:
        {
            std::unreachable();
            break;
        }
    }

    // Destination Register inherits rd's type, trusts the programmer that the immediate was meaningfully encoded to
    // the corresponding type. Sworry I couldn't put a tag field in the immediate
    result |= (static_cast<uint64_t>(type) << 35) & MASK_41; // This is inserting rs1's type into the result;
    vm.GPR.at(rd) = static_cast<uint64_t>(result);
}

void execute_c_type(BVM &vm, const InstructionInfo instruction);
void execute_j_type(BVM &vm, const InstructionInfo instruction);

void execute(BVM &vm, const InstructionInfo& instruction)
{

    switch (instruction.group)
    {

        case InstructionGroup::R_TYPE:
        {
            execute_r_type(vm, instruction);
            break;
        }
        case InstructionGroup::C_TYPE:
        {
            execute_c_type(vm, instruction);
            break;
        }
        case InstructionGroup::I_TYPE:
        {
            execute_i_type(vm, instruction);
        }
    }
}

void emulate(BVM &vm, std::vector<std::pair<string, size_t>> &instruction_buffer)
{
    while (vm.PC < instruction_buffer.size())
    {
        print_status(vm);
        auto fetched_instruction = instruction_buffer.at(vm.PC);
        auto decoded_instruction = decode(fetched_instruction);
        execute(vm, decoded_instruction);
    }
    print_status(vm);
}

int main(int argc, char *argv[])
{
    // vector<string> input_args(argv + 1, argv + argc);

    assert(argc >= 2);
    vector<std::pair<string, size_t>> instruction_buffer = load_hex(argv[1]);
    assert(instruction_buffer.size() > 0);

    BVM vm{};
    emulate(vm, instruction_buffer);
    
}
