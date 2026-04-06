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
#include <utility>
#include <vector>

using std::ifstream;
using std::string;
using std::vector;

// const uint64_t MASK_41 = (1ULL << 41) - 1;
// const uint64_t UMAX_41 = (1ULL << 41) - 1;
// const int64_t SMAX_41 = (1ULL << 40) - 1;
// const uint64_t MASK_35 = (1ULL << 35) - 1;
const uint64_t MASK_46 = (1ULL << 46) - 1;
const uint64_t UMAX_40BIT = (1ULL << 40) - 1;
const int64_t SMAX_40BIT = (1LL << 39) - 1;
const int64_t SMIN_40BIT = -(1LL << 39);

constexpr uint64_t extract_bits(const uint64_t &val, uint64_t high, uint64_t low)
{
    uint64_t width = high - low + 1ULL;
    uint64_t mask = (1ULL << width) - 1ULL;
    uint64_t ret = (val >> low) & mask;
    return ret;
}

constexpr TypeLabels extract_tag(const uint64_t &val)
{
    uint64_t width = TAG_FIELD_HIGH - TAG_FIELD_LOW + 1ULL;
    uint64_t mask = (1ULL << width) - 1ULL;
    auto ret = static_cast<TypeLabels>((val >> TAG_FIELD_LOW) & mask);
    return ret;
}

constexpr uint64_t extract_data(const uint64_t &val)
{
    uint64_t width = DATA_FIELD_HIGH - DATA_FIELD_LOW + 1ULL;
    uint64_t mask = (1ULL << width) - 1ULL;
    uint64_t ret = (val >> DATA_FIELD_LOW) & mask;
    return ret;
}

constexpr InstructionLabel extract_opcode(const uint64_t &val)
{
    uint64_t width = OPCODE_FIELD_HIGH - OPCODE_FIELD_LOW + 1ULL;
    uint64_t mask = (1ULL << width) - 1ULL;
    auto ret = static_cast<InstructionLabel>((val >> OPCODE_FIELD_LOW) & mask);
    return ret;
}

constexpr uint64_t extract_imm(const uint64_t &val)
{
    uint64_t width = IMM_FIELD_HIGH - IMM_FIELD_LOW + 1ULL;
    uint64_t mask = (1ULL << width) - 1ULL;
    uint64_t ret = ((val >> IMM_FIELD_LOW) & mask);
    return ret;
}

constexpr uint64_t convert(const TypeLabels SOURCE, const TypeLabels TARGET, const uint64_t &VAL, BVM &VM)
{

    auto retag = [&](const uint64_t VAL) -> uint64_t
    {
        const uint64_t RETAGGED_TYPE = static_cast<uint64_t>(static_cast<uint64_t>(std::to_underlying(TARGET)) << 40ULL);
        const uint64_t VAL_DATA_RETAGGED = extract_data(VAL) | RETAGGED_TYPE;
        return VAL_DATA_RETAGGED;
    };
    
    auto scale = [&](const uint64_t VAL)
    {
        const uint64_t INT_BITS = [&]() -> uint64_t
        {
            switch (SOURCE)
            {
                case TypeLabels::Q32_8:
                    return 32;
                case TypeLabels::Q24_16:
                    return 24;
                case TypeLabels::Q8_32:
                    return 8;
                default:
                    throw std::runtime_error("Not a valid source type for clamping?");
            }
        }();
        const uint64_t FRAC_BITS = DATA_FIELD_SIZE - INT_BITS;
        uint64_t ret;
        switch (TARGET)
        {

            case TypeLabels::UINT:
            {
                ret = extract_data(VAL) >> FRAC_BITS;
                ret |= std::to_underlying(TARGET) << 40ULL;
                break;
            }
            case TypeLabels::UMOD:
            {
            }
            case TypeLabels::SINT:
            {
            }
            case TypeLabels::SMOD:
            {
            }
            case TypeLabels::Q32_8:
            {
            }
            case TypeLabels::Q24_16:
            {
            }
            case TypeLabels::Q8_32:
            {
            }
            default:
                throw std::runtime_error("Not a valid target type for clamping?");
        }

        VM.STATUS_REGS[SATURATION_FLAG] = true;
    };

    auto clamp = []() {

    };
    auto trunc = []() {

    };
    auto numToBool = []() {

    };
    auto invalid = []() {

    };

    const ConvOp CONVERSION_FUNC = CONV_TABLE[std::to_underlying(SOURCE)][std::to_underlying(TARGET)];
    uint64_t result;

    switch (CONVERSION_FUNC)
    {
        case ConvOp::Id:
        {
            result = VAL;
            break;
        }
        case ConvOp::Retag:
        {
            result = retag(VAL);
            break;
        }
        case ConvOp::Zero:
        {
            result = 0;
            break;
        }
        case ConvOp::Scale:
        {
            result = scale(VAL);
            break;
        }
        case ConvOp::Clamp:
        {
            break;
        }
        case ConvOp::Trunc:
        {
            break;
        }
        case ConvOp::NonZero:
        {
            break;
        }
        case ConvOp::BoolTo:
        {
            break;
        }
        case ConvOp::Invalid:
        {
            break;
        }
    }

    return result;
}

void print_status(BVM &vm, std::ofstream &logfile)
{

    std::println(logfile, "==========================================================");
    std::println(logfile, "‖ {:1} {:>20} ‖ {:15} {:08b} ‖", "PC: ", vm.PC, "Status Register: ", vm.STATUS_REGS.to_ullong());
    std::println(logfile, "=====================================================================================");
    for (size_t i = 0; i < vm.GPR.size(); i++)
    {
        TypeLabels type = extract_tag(vm.GPR[i]);
        if (type >= TypeLabels::SINT && type <= TypeLabels::SPACKED)
        {
            uint64_t val = extract_data(vm.GPR[i]);
            std::println(logfile, "‖ {:1} {:2}: {:>8} {:046b} | {:>15} ‖", "GPR", i, TYPELABEL_TO_STRING.at(type), vm.GPR.at(i),
                         val);
        }
        else
        {
            uint64_t val = extract_data(vm.GPR[i]);
            std::println(logfile, "‖ {:1} {:2}: {:>8} {:046b} | {:>15} ‖", "GPR", i, TYPELABEL_TO_STRING.at(type), vm.GPR.at(i),
                         val);
        }
    }
    std::println(logfile, "=====================================================================================\n");
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

    const TypeLabels instruction_tag_field = extract_tag(curr_instruction_bin);
    if (instruction_tag_field != TypeLabels::INSTRUCTION)
        throw std::invalid_argument(std::format("Not an intstruction type at line number: {}", curr_line_num));

    const InstructionLabel opcode_field = extract_opcode(curr_instruction_bin);

    auto it = LABEL_TO_INFOSTRUCT.find(opcode_field);
    if (it == LABEL_TO_INFOSTRUCT.end())
        throw std::invalid_argument(std::format("Invalid opcode: {:B}, not found in lookup table at line {}",
                                                std::to_underlying(opcode_field), curr_line_num));

    instruction_struct = LABEL_TO_INFOSTRUCT.at(opcode_field);
    instruction_struct.bin = curr_instruction_bin;
    instruction_struct.line_num = curr_line_num;

    return instruction_struct;
}

void execute_r_type(BVM &vm, const InstructionInfo &instruction)
{

    constexpr uint64_t RD_HIGH = 31;
    constexpr uint64_t RD_LOW = 27;
    constexpr uint64_t RS1_LOW = 22;
    constexpr uint64_t RS1_HIGH = 26;
    constexpr uint64_t RS2_LOW = 17;
    constexpr uint64_t RS2_HIGH = 21;

    const uint64_t INSTRUCTION_BIN = instruction.bin;
    const uint64_t RD = extract_bits(INSTRUCTION_BIN, RD_HIGH, RD_LOW);
    const uint64_t RS1 = extract_bits(INSTRUCTION_BIN, RS1_HIGH, RS1_LOW);
    const uint64_t RS2 = extract_bits(INSTRUCTION_BIN, RS2_HIGH, RS2_LOW);

    if (RD > MAX_GPR_RANGE || RD == 0)
        throw std::runtime_error(std::format("Destination Register out of Range or writing to Zero Register: {}", RD));
    if (RS1 > MAX_GPR_RANGE) throw std::runtime_error(std::format("RS1 Register out of Range: {}", RS1));
    if (RS2 > MAX_GPR_RANGE) throw std::runtime_error(std::format("RS2 Register out of Range: {}", RS2));

    const uint64_t RS1_BIN = vm.GPR[RS1];
    const uint64_t RS2_BIN = vm.GPR[RS2];

    const uint64_t RS1_VAL = extract_data(RS1_BIN);
    const uint64_t RS2_VAL = extract_data(RS2_BIN);

    const TypeLabels RS1_TYPE = extract_tag(RS1_BIN);
    const TypeLabels RS2_TYPE = extract_tag(RS2_BIN);

    if (RS1_TYPE != RS2_TYPE)
        throw std::runtime_error(std::format("Register types do not match!\nRS1 is: {}\nRS2 is: {}", RS2,
                                             TYPELABEL_TO_STRING.at(RS1_TYPE), TYPELABEL_TO_STRING.at(RS2_TYPE)));

    uint64_t result = 0;
    switch (instruction.label)
    {

        case InstructionLabel::ADD:
        {
            result = RS1_VAL + RS2_VAL;
            break;
        }
        case InstructionLabel::SUB:
        {
            result = RS1_VAL - RS2_VAL;
            break;
        }
        case InstructionLabel::XOR:
        {
            result = RS1_VAL ^ RS2_VAL;
            break;
        }
        case InstructionLabel::OR:
        {
            result = RS1_VAL | RS2_VAL;
            break;
        }
        case InstructionLabel::AND:
        {
            result = RS1_VAL & RS2_VAL;
            break;
        }
        case InstructionLabel::SLL:
        {
            result = RS1_VAL << RS2_VAL;
            break;
        }
        case InstructionLabel::SHR:
        {
            // TODO
            break;
        }
        case InstructionLabel::MUL:
        {
            result = RS1_VAL * RS2_VAL;
            break;
        }
        case InstructionLabel::MULH:
        {
            // TODO Change this to DIV
            break;
        }
        case InstructionLabel::MAC:
        {
            const uint64_t RD_BIN = vm.GPR[RD];
            const TypeLabels RD_TYPE = extract_tag(RD_BIN);
            if (RD_TYPE != RS1_TYPE)
            {
                throw std::runtime_error(std::format("Register types do not match for MAC!\nRS1 is: {}\nRS2 is: {}\nRD is: {}",
                                                     RS2, TYPELABEL_TO_STRING.at(RS1_TYPE), TYPELABEL_TO_STRING.at(RS2_TYPE),
                                                     TYPELABEL_TO_STRING.at(RD_TYPE)));
            }
            const uint64_t RD_VAL = extract_data(RD_BIN);
            result = RD_VAL + (RS1_VAL * RS2_VAL);
            break;
        }
        default:
        {
            std::unreachable();
            break;
        }
    }
    if ((RS1_TYPE >= TypeLabels::UINT && RS1_TYPE <= TypeLabels::BOOL) && RS1_TYPE != TypeLabels::UMOD)
    {
        uint64_t result_min = std::min(result, UMAX_40BIT);
        if (result_min != result || result == UMAX_40BIT) vm.STATUS_REGS[SATURATION_FLAG] = true;
        result = result_min;
    }
    else if ((RS1_TYPE >= TypeLabels::SINT && RS1_TYPE <= TypeLabels::SPACKED) && RS1_TYPE != TypeLabels::SMOD)
    {
        int64_t RESULT_SIGNED = static_cast<int64_t>(result);
        int64_t result_clamped = std::clamp(RESULT_SIGNED, SMIN_40BIT, SMAX_40BIT);
        if (result_clamped != RESULT_SIGNED || result_clamped == SMIN_40BIT || result_clamped == SMAX_40BIT)
            vm.STATUS_REGS[SATURATION_FLAG] = true;
        result = static_cast<uint64_t>(result_clamped);
    }
    result |= (static_cast<uint64_t>(RS1_TYPE) << 40ULL);
    result &= MASK_46;
    vm.GPR.at(RD) = result;
    vm.PC += 1;
}
void execute_i_type(BVM &vm, const InstructionInfo &instruction)
{
    // Register Numbers
    constexpr uint64_t RD_HIGH = 31;
    constexpr uint64_t RD_LOW = 27;

    constexpr uint64_t RS1_HIGH = 26;
    constexpr uint64_t RS1_LOW = 22;

    const uint64_t RD = extract_bits(instruction.bin, RD_HIGH, RD_LOW);
    const uint64_t RS1 = extract_bits(instruction.bin, RS1_HIGH, RS1_LOW);
    // Immediate/Register Total Values (Including type)
    const uint64_t RS1_BIN = vm.GPR.at(RS1);
    const TypeLabels RS1_TYPE = extract_tag(RS1_BIN);
    uint64_t rs1_val = extract_data(RS1_BIN);
    uint64_t imm_val = extract_imm(instruction.bin);

    if (RS1_TYPE >= TypeLabels::INERT || RS1_TYPE == TypeLabels::FREE)
        throw std::invalid_argument(std::format("Not a valid type of operand register source [1] [{:}] at line [{:X}]",
                                                TYPELABEL_TO_STRING.at(RS1_TYPE), instruction.line_num));

    if (const TypeLabels type_dest = extract_tag(vm.GPR.at(RD)); type_dest >= TypeLabels::INERT)
        throw std::invalid_argument(
            std::format("Not a valid type of register destimation, is inert at line {:X}", instruction.line_num));

    uint64_t result = 0;

    if (RS1_TYPE >= TypeLabels::SINT && RS1_TYPE <= TypeLabels::SPACKED)
    {
        constexpr uint64_t PADDING_BITS_RS1 = 24;
        constexpr uint64_t PADDING_BITS_IMM = 24;
        // Sign Extend Instead
        rs1_val = static_cast<uint64_t>(((static_cast<int64_t>(rs1_val) << PADDING_BITS_RS1) >> PADDING_BITS_RS1));
        imm_val = static_cast<uint64_t>((static_cast<int64_t>(imm_val) << PADDING_BITS_IMM) >> PADDING_BITS_IMM);
    }

    switch (instruction.label)
    {
        case InstructionLabel::ADDI:
        {
            result = rs1_val + imm_val;

            // Saturation Check
            if ((RS1_TYPE >= TypeLabels::SINT && RS1_TYPE <= TypeLabels::SPACKED) && RS1_TYPE != TypeLabels::SMOD)
            {
                auto result_clamped = static_cast<uint64_t>(std::clamp(static_cast<int64_t>(result), SMAX_40BIT, SMIN_40BIT));
                if (result_clamped != result) vm.STATUS_REGS[SATURATION_FLAG] = true;
                result = result_clamped;
                // int64_t rs1_val_signed = static_cast<int64_t>(rs1_val);
                // int64_t imm_signed = static_cast<int64_t>(rs1_val);
                // if ((rs1_val_signed > 0 && imm_signed > 0) && result < 0)
                //{
                //  Two Postives become negative, overflow
                //    result = SMAX_40BIT;
                //}
                // else if (((rs1_val_signed < 0) && (imm_signed < 0)) && result > 0)
                //{
                // Two negatives become postive, underflow
                //  result = static_cast<uint64_t>(SMIN_40BIT);
                //}
            }
            else if ((RS1_TYPE >= TypeLabels::UINT && RS1_TYPE <= TypeLabels::BOOL) && RS1_TYPE != TypeLabels::UMOD)
            {
                // Overflow saturate
                if (result > UMAX_40BIT)
                {
                    result = UMAX_40BIT;
                    vm.STATUS_REGS[SATURATION_FLAG] = true;
                }
            }
            break;
        }
        case InstructionLabel::XORI:
        {
            result = rs1_val ^ imm_val;
            break;
        }
        case InstructionLabel::ORI:
        {
            result = rs1_val | imm_val;
            break;
        }
        case InstructionLabel::ANDI:
        {
            result = rs1_val & imm_val;
            break;
        }
        case InstructionLabel::SLLI:
        {
            result = rs1_val << imm_val;
            break;
        }
        case InstructionLabel::SHRI:
        {
            // TODO
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
    vm.PC += 1;
    result |= (static_cast<uint64_t>(RS1_TYPE) << TAG_FIELD_LOW) & MASK_46; // This is inserting rs1's type into the result;
    vm.GPR.at(RD) = static_cast<uint64_t>(result);                          // NOLINT
}

void execute_c_type(BVM &vm, const InstructionInfo &instruction)
{
    constexpr uint64_t TARGET_TYPE_HIGH = 21;
    constexpr uint64_t TARGET_TYPE_LOW = 14;

    constexpr uint64_t RD_HIGH = 31;
    constexpr uint64_t RD_LOW = 27;

    constexpr uint64_t RS1_HIGH = 26;
    constexpr uint64_t RS1_LOW = 22;

    const uint64_t RD = extract_bits(instruction.bin, RD_HIGH, RD_LOW);
    const uint64_t RS1 = extract_bits(instruction.bin, RS1_HIGH, RS1_LOW);
    const TypeLabels TARGET_TYPE = static_cast<TypeLabels>(extract_bits(instruction.bin, TARGET_TYPE_HIGH, TARGET_TYPE_LOW));

    if (RD > MAX_GPR_RANGE || RD == 0)
        throw std::runtime_error(std::format("Destination Register out of Range or writing to Zero Register: {}", RD));
    if (RS1 > MAX_GPR_RANGE) throw std::runtime_error(std::format("RS1 Register out of Range: {}", RS1));
    if (const TypeLabels type_dest = extract_tag(vm.GPR.at(RD)); type_dest >= TypeLabels::INERT)
        throw std::invalid_argument(
            std::format("Not a valid type of register destimation, is inert at line {:X}", instruction.line_num));

    const uint64_t RS1_BIN = vm.GPR.at(RS1);
    const TypeLabels RS1_TYPE = extract_tag(RS1_BIN);
    const uint64_t RS1_VAL = extract_data(RS1_BIN);

    ConversionPairing pair = {.source = RS1_TYPE, .target = TARGET_TYPE};

    const uint64_t RS1_VAL_CONVERTED = convert(pair, RS1_VAL);
}
void execute_j_type(BVM &vm, const InstructionInfo &instruction)
{
    throw std::runtime_error(
        std::format("NOT IMPLEMENTED YET, PC at {}, Instruction is {}", vm.PC, LABEL_TO_STRING.at(instruction.label)));
}

void execute(BVM &vm, const InstructionInfo &instruction)
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
    std::ofstream logfile("log.txt");
    if (!logfile.is_open()) { throw std::invalid_argument("Could not open log file!"); }

    while (vm.PC < instruction_buffer.size())
    {
        print_status(vm, logfile);
        auto fetched_instruction = instruction_buffer.at(vm.PC);
        auto decoded_instruction = decode(fetched_instruction);
        execute(vm, decoded_instruction);
    }
    assert(extract_data(vm.GPR[0]) == 0);
    print_status(vm, logfile);
    logfile.close();
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
