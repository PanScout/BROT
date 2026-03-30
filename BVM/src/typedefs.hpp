#pragma once
#include <array>
#include <cstddef>
#include <cstdint>
#include <map>
#include <set>
#include <string>
#include <unordered_map>
#include <unordered_set>

enum class InstructionGroup
{
    R_TYPE,
    C_TYPE,
    I_TYPE
};

enum class ArithemeticFlag
{
    OVERFLOW_F,
    UNDERFLOW_F,
    OK
};

enum class DataType : uint8_t
{
    Free = 0x00,
    Uint = 0x01,
    Sint = 0x02,
    Umod = 0x03,
    Smod = 0x04,
    Q28_7 = 0x05,
    Q18_17 = 0x06,
    Q7_28 = 0x07,
    PtrSingle = 0x08,
    PtrArray = 0x09,
    PtrEnd = 0x0A,
    Bool = 0x0B,
    Instr = 0x0C,
    Crumb5x7u = 0x0D,
    Crumb5x7s = 0x0E,
    Inert = 0x0F,
};

enum class InstructionLabel : uint8_t
{
    // R-type
    ADD = 0x00,
    SUB = 0x01,
    XOR = 0x02,
    OR = 0x03,
    AND = 0x04,
    SLL = 0x05,
    SRL = 0x06,
    SRA = 0x07,
    MUL = 0x08,
    MULH = 0x09,
    MAC = 0x0A,
    // I-type
    ADDI = 0x12,
    XORI = 0x13,
    ORI = 0x14,
    ANDI = 0x15,
    SLLI = 0x16,
    SRLI = 0x17,
    SRAI = 0x18,
    // C-type
    CONVERT = 0x1F,
};

struct InstructionInfo
{
    InstructionLabel label{};
    InstructionGroup group{};
    std::string str_label;
    uint64_t bin = 0;
    size_t operand_count = 0;
    size_t line_num = 0;
};

struct PointerReg
{
    uint64_t upper;
    uint64_t lower;
};

enum class ArthimeticFlags : uint8_t
{
    OVERFLOW_F,
    UNDERFLOW_F,
    OK_F,
};

inline const std::set<uint8_t> VALID_OPCODES = {
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x1F,
};

inline const std::map<InstructionLabel, InstructionInfo> LABEL_TO_INFOSTRUCT = {
    {InstructionLabel::ADD,
     {.label = InstructionLabel::ADD, .group = InstructionGroup::R_TYPE, .str_label = "ADD", .operand_count = 3}},
    {InstructionLabel::SUB,
     {.label = InstructionLabel::SUB, .group = InstructionGroup::R_TYPE, .str_label = "SUB", .operand_count = 3}},
    {InstructionLabel::XOR,
     {.label = InstructionLabel::XOR, .group = InstructionGroup::R_TYPE, .str_label = "XOR", .operand_count = 3}},
    {InstructionLabel::OR,
     {.label = InstructionLabel::OR, .group = InstructionGroup::R_TYPE, .str_label = "OR", .operand_count = 3}},
    {InstructionLabel::AND,
     {.label = InstructionLabel::AND, .group = InstructionGroup::R_TYPE, .str_label = "AND", .operand_count = 3}},
    {InstructionLabel::SLL,
     {.label = InstructionLabel::SLL, .group = InstructionGroup::R_TYPE, .str_label = "SLL", .operand_count = 3}},
    {InstructionLabel::SRL,
     {.label = InstructionLabel::SRL, .group = InstructionGroup::R_TYPE, .str_label = "SRL", .operand_count = 3}},
    {InstructionLabel::SRA,
     {.label = InstructionLabel::SRA, .group = InstructionGroup::R_TYPE, .str_label = "SRA", .operand_count = 3}},
    {InstructionLabel::MUL,
     {.label = InstructionLabel::MUL, .group = InstructionGroup::R_TYPE, .str_label = "MUL", .operand_count = 3}},
    {InstructionLabel::MULH,
     {.label = InstructionLabel::MULH, .group = InstructionGroup::R_TYPE, .str_label = "MULH", .operand_count = 3}},
    {InstructionLabel::MAC,
     {.label = InstructionLabel::MAC, .group = InstructionGroup::R_TYPE, .str_label = "MAC", .operand_count = 3}},
    {InstructionLabel::ADDI,
     {.label = InstructionLabel::ADDI, .group = InstructionGroup::I_TYPE, .str_label = "ADDI", .operand_count = 3}},
    {InstructionLabel::XORI,
     {.label = InstructionLabel::XORI, .group = InstructionGroup::I_TYPE, .str_label = "XORI", .operand_count = 3}},
    {InstructionLabel::ORI,
     {.label = InstructionLabel::ORI, .group = InstructionGroup::I_TYPE, .str_label = "ORI", .operand_count = 3}},
    {InstructionLabel::ANDI,
     {.label = InstructionLabel::ANDI, .group = InstructionGroup::I_TYPE, .str_label = "ANDI", .operand_count = 3}},
    {InstructionLabel::SLLI,
     {.label = InstructionLabel::SLLI, .group = InstructionGroup::I_TYPE, .str_label = "SLLI", .operand_count = 3}},
    {InstructionLabel::SRLI,
     {.label = InstructionLabel::SRLI, .group = InstructionGroup::I_TYPE, .str_label = "SRLI", .operand_count = 3}},
    {InstructionLabel::SRAI,
     {.label = InstructionLabel::SRAI, .group = InstructionGroup::I_TYPE, .str_label = "SRAI", .operand_count = 3}},
    {InstructionLabel::CONVERT,
     {.label = InstructionLabel::CONVERT, .group = InstructionGroup::C_TYPE, .str_label = "CONVERT", .operand_count = 3}},
};

inline const std::unordered_map<uint8_t, std::string> UINT_TO_TYPE_STRING = {
    {0x00, "free"},   {0x01, "uint"},      {0x02, "sint"},       {0x03, "umod"},      {0x04, "smod"},    {0x05, "q28.7"},
    {0x06, "q18.17"}, {0x07, "q7.28"},     {0x08, "ptr_single"}, {0x09, "ptr_array"}, {0x0A, "ptr_end"}, {0x0B, "bool"},
    {0x0C, "instr"},  {0x0D, "crumb5x7u"}, {0x0E, "crumb5x7s"},  {0x0F, "inert"},
};
inline const std::unordered_map<std::string, uint8_t> TYPE_STRING_TO_UINT = {
    {"free", 0x00},   {"uint", 0x01},      {"sint", 0x02},       {"umod", 0x03},      {"smod", 0x04},    {"q28.7", 0x05},
    {"q18.17", 0x06}, {"q7.28", 0x07},     {"ptr_single", 0x08}, {"ptr_array", 0x09}, {"ptr_end", 0x0A}, {"bool", 0x0B},
    {"instr", 0x0C},  {"crumb5x7u", 0x0D}, {"crumb5x7s", 0x0E},  {"inert", 0x0F},
};

inline const std::unordered_map<InstructionLabel, std::string> LABEL_TO_STRING = {
    // R-type
    {InstructionLabel::ADD, "ADD"},
    {InstructionLabel::SUB, "SUB"},
    {InstructionLabel::XOR, "XOR"},
    {InstructionLabel::OR, "OR"},
    {InstructionLabel::AND, "AND"},
    {InstructionLabel::SLL, "SLL"},
    {InstructionLabel::SRL, "SRL"},
    {InstructionLabel::SRA, "SRA"},
    {InstructionLabel::MUL, "MUL"},
    {InstructionLabel::MULH, "MULH"},
    {InstructionLabel::MAC, "MAC"},
    // I-type
    {InstructionLabel::ADDI, "ADDI"},
    {InstructionLabel::XORI, "XORI"},
    {InstructionLabel::ORI, "ORI"},
    {InstructionLabel::ANDI, "ANDI"},
    {InstructionLabel::SLLI, "SLLI"},
    {InstructionLabel::SRLI, "SRLI"},
    {InstructionLabel::SRAI, "SRAI"},
    // C-type
    {InstructionLabel::CONVERT, "CONVERT"},
};

inline const std::unordered_map<DataType, std::string> DATATYPE_TO_STRING = {
    {DataType::Free, "free"},          {DataType::Uint, "uint"},           {DataType::Sint, "sint"},
    {DataType::Umod, "umod"},          {DataType::Smod, "smod"},           {DataType::Q28_7, "q28.7"},
    {DataType::Q18_17, "q18.17"},      {DataType::Q7_28, "q7.28"},         {DataType::PtrSingle, "ptr_single"},
    {DataType::PtrArray, "ptr_array"}, {DataType::PtrEnd, "ptr_end"},      {DataType::Bool, "bool"},
    {DataType::Instr, "instr"},        {DataType::Crumb5x7u, "crumb5x7u"}, {DataType::Crumb5x7s, "crumb5x7s"},
    {DataType::Inert, "inert"},
};

inline const std::unordered_set<DataType> UNSIGNED_TYPES = {
    DataType::Uint,
    DataType::Umod,
    DataType::Crumb5x7u,
};

inline const std::unordered_set<DataType> SIGNED_TYPES = {
    DataType::Sint, DataType::Smod, DataType::Q18_17, DataType::Q28_7, DataType::Q7_28, DataType::Crumb5x7s,
};

struct BVM
{
    size_t PC = 0;
    std::array<uint64_t, 23> GPR{};
    std::array<PointerReg, 13> PR{};
    std::array<uint64_t, 8192> MEM{};

    BVM()
    {
        GPR[0] |= 1ULL << 35; // Write UINT to register 0
    }
};


inline constexpr uint64_t START_TAG_FIELD = 40;
inline constexpr uint64_t END_TAG_FIELD = 45;

inline constexpr uint64_t START_OPCODE_FIELD = 32;
inline constexpr uint64_t END_OPCODE_FIELD = 39;


inline constexpr uint64_t I_TYPE_START_REGISTER_FIELD = 22;
inline constexpr uint64_t I_TYPE_END_REGISTER_FIELD = 31;
inline constexpr uint64_t I_TYPE_REGISTER_FIELD_WIDTH = 5;
inline constexpr uint64_t I_TYPE_IMM_FIELD_START = 0;
inline constexpr uint64_t I_TYPE_IMM_FIELD_END = 21;
