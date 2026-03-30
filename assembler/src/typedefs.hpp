#pragma once
#include <cstddef>
#include <cstdint>
#include <string>
#include <unordered_map>
#include <vector>

enum class InstructionGroup : uint8_t
{
    R_TYPE, // Register to Register
    S_TYPE, // Store to memory
    B_TYPE, // Branch instructions
    I_TYPE, // Immediate instructions
    U_TYPE, // Upper immediate loading
    J_TYPE, // Jump instructions
    C_TYPE, // Convert instructions
    L_TYPE, // Loop instructions
    P_TYPE, // Pointer instructions (MKPTR, PNARROW, pointer store/load)
    Q_TYPE  // System / query instructions (RDSR, PLEN, MTAG)
};

enum class TypeLabels : uint8_t
{
    FREE = 0x00,
    UINT = 0x01, // <- Unsigned Start
    UMOD = 0x02,
    UPACKED = 0x03,
    BOOL = 0x04, // <- Unsigned End
    SINT = 0x05, // <- Signed Start
    SMOD = 0x06,
    Q32_8 = 0x07,
    Q24_16 = 0x08,
    Q8_32 = 0x09,
    SPACKED = 0x0A,     // <- Signed End
    INSTRUCTION = 0x0B, // <- System Start
    PTR = 0x0C,
    INERT = 0x0D, // <- System End
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
    SHR = 0x06,
    MUL = 0x07,
    MULH = 0x08,
    MAC = 0x09,

    // I-type
    ADDI = 0x11,
    XORI = 0x12,
    ORI = 0x13,
    ANDI = 0x14,
    SLLI = 0x15,
    SHRI = 0x16,

    // C-type
    CONVERT = 0x1D,
};

struct InstructionInfo
{
    InstructionLabel label{};
    InstructionGroup group{};
    std::string instruction_label_str;
    std::vector<std::string> operands_list;
    size_t line_num{};
};

inline const std::unordered_map<std::string, InstructionInfo> STRING_TO_INFOSTRUCT = {
    {"ADD",
     {.label = InstructionLabel::ADD, .group = InstructionGroup::R_TYPE, .instruction_label_str = "ADD", .operands_list = {}}},
    {"SUB",
     {.label = InstructionLabel::SUB, .group = InstructionGroup::R_TYPE, .instruction_label_str = "SUB", .operands_list = {}}},
    {"XOR",
     {.label = InstructionLabel::XOR, .group = InstructionGroup::R_TYPE, .instruction_label_str = "XOR", .operands_list = {}}},
    {"OR",
     {.label = InstructionLabel::OR, .group = InstructionGroup::R_TYPE, .instruction_label_str = "OR", .operands_list = {}}},
    {"AND",
     {.label = InstructionLabel::AND, .group = InstructionGroup::R_TYPE, .instruction_label_str = "AND", .operands_list = {}}},
    {"SLL",
     {.label = InstructionLabel::SLL, .group = InstructionGroup::R_TYPE, .instruction_label_str = "SLL", .operands_list = {}}},
    {"SHR",
     {.label = InstructionLabel::SHR, .group = InstructionGroup::R_TYPE, .instruction_label_str = "SHR", .operands_list = {}}},
    {"MUL",
     {.label = InstructionLabel::MUL, .group = InstructionGroup::R_TYPE, .instruction_label_str = "MUL", .operands_list = {}}},
    {"MULH",
     {.label = InstructionLabel::MULH, .group = InstructionGroup::R_TYPE, .instruction_label_str = "MULH", .operands_list = {}}},
    {"MAC",
     {.label = InstructionLabel::MAC, .group = InstructionGroup::R_TYPE, .instruction_label_str = "MAC", .operands_list = {}}},
    {"ADDI",
     {.label = InstructionLabel::ADDI, .group = InstructionGroup::I_TYPE, .instruction_label_str = "ADDI", .operands_list = {}}},
    {"XORI",
     {.label = InstructionLabel::XORI, .group = InstructionGroup::I_TYPE, .instruction_label_str = "XORI", .operands_list = {}}},
    {"ORI",
     {.label = InstructionLabel::ORI, .group = InstructionGroup::I_TYPE, .instruction_label_str = "ORI", .operands_list = {}}},
    {"ANDI",
     {.label = InstructionLabel::ANDI, .group = InstructionGroup::I_TYPE, .instruction_label_str = "ANDI", .operands_list = {}}},
    {"SLLI",
     {.label = InstructionLabel::SLLI, .group = InstructionGroup::I_TYPE, .instruction_label_str = "SLLI", .operands_list = {}}},
    {"SHRI",
     {.label = InstructionLabel::SHRI, .group = InstructionGroup::I_TYPE, .instruction_label_str = "SHRI", .operands_list = {}}},
    {"CONVERT",
     {.label = InstructionLabel::CONVERT,
      .group = InstructionGroup::C_TYPE,
      .instruction_label_str = "CONVERT",
      .operands_list = {}}},
};

inline const std::unordered_map<TypeLabels, std::string> TYPELABEL_TO_STRING = {
    {TypeLabels::FREE, "free"},   {TypeLabels::UINT, "uint"},   {TypeLabels::UMOD, "umod"},
    {TypeLabels::UPACKED, "pku"}, {TypeLabels::BOOL, "bool"},   {TypeLabels::SINT, "sint"},
    {TypeLabels::SMOD, "smod"},   {TypeLabels::Q32_8, "q32.8"}, {TypeLabels::Q24_16, "q24.16"},
    {TypeLabels::Q8_32, "q8.32"}, {TypeLabels::SPACKED, "pks"}, {TypeLabels::INSTRUCTION, "instr"},
    {TypeLabels::PTR, "ptr"},     {TypeLabels::INERT, "inert"},
};

inline const std::unordered_map<std::string, TypeLabels> STRING_TO_TYPELABEL = {
    {"free", TypeLabels::FREE},   {"uint", TypeLabels::UINT},   {"umod", TypeLabels::UMOD},
    {"pku", TypeLabels::UPACKED}, {"bool", TypeLabels::BOOL},   {"sint", TypeLabels::SINT},
    {"smod", TypeLabels::SMOD},   {"q32.8", TypeLabels::Q32_8}, {"q24.16", TypeLabels::Q24_16},
    {"q8.32", TypeLabels::Q8_32}, {"pks", TypeLabels::SPACKED}, {"instr", TypeLabels::INSTRUCTION},
    {"ptr", TypeLabels::PTR},     {"inert", TypeLabels::INERT},
};

inline constexpr uint64_t START_TAG_FIELD = 40;
inline constexpr uint64_t END_TAG_FIELD = 45;

inline constexpr uint64_t START_OPCODE_FIELD = 32;
inline constexpr uint64_t END_OPCODE_FIELD = 39;

inline constexpr uint64_t R_TYPE_START_REGISTER_FIELD = 17;
inline constexpr uint64_t R_TYPE_END_REGISTER_FIELD = 31;
inline constexpr uint64_t R_TYPE_REGISTER_FIELD_WIDTH = 5;

inline constexpr uint64_t C_TYPE_START_REGISTER_FIELD = 22;
inline constexpr uint64_t C_TYPE_END_REGISTER_FIELD = 31;
inline constexpr uint64_t C_TYPE_REGISTER_FIELD_WIDTH = 5;
inline constexpr uint64_t C_TYPE_START_TARGET_TYPE_FIELD = 14;

inline constexpr uint64_t I_TYPE_START_REGISTER_FIELD = 22;
inline constexpr uint64_t I_TYPE_END_REGISTER_FIELD = 31;
inline constexpr uint64_t I_TYPE_REGISTER_FIELD_WIDTH = 5;
inline constexpr uint64_t I_TYPE_IMM_FIELD_START = 0;
inline constexpr uint64_t I_TYPE_IMM_FIELD_END = 21;

inline constexpr uint64_t UIMM_MAX = (1ULL << 22) - 1;
inline constexpr int64_t SIMM_MAX = (1LL << 21) - 1;
inline constexpr int64_t SIMM_MIN = -((1LL << 21));
inline constexpr uint64_t MASK_IMM22 = (1ULL << 22) - 1;
