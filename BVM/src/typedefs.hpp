#pragma once
#include <array>
#include <bitset>
#include <cstddef>
#include <cstdint>
#include <map>
#include <string>

enum class TypeLabels : uint64_t
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

inline const std::map<TypeLabels, std::string> TYPELABEL_TO_STRING = {
    {TypeLabels::FREE, "free"},   {TypeLabels::UINT, "uint"},   {TypeLabels::UMOD, "umod"},
    {TypeLabels::UPACKED, "pku"}, {TypeLabels::BOOL, "bool"},   {TypeLabels::SINT, "sint"},
    {TypeLabels::SMOD, "smod"},   {TypeLabels::Q32_8, "q32.8"}, {TypeLabels::Q24_16, "q24.16"},
    {TypeLabels::Q8_32, "q8.32"}, {TypeLabels::SPACKED, "pks"}, {TypeLabels::INSTRUCTION, "instr"},
    {TypeLabels::PTR, "ptr"},     {TypeLabels::INERT, "inert"},
};

enum class InstructionGroup
{
    R_TYPE,
    C_TYPE,
    I_TYPE
};

enum class InstructionLabel : uint64_t
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


inline const std::map<InstructionLabel, InstructionInfo> LABEL_TO_INFOSTRUCT = {
    {InstructionLabel::ADD, {.label = InstructionLabel::ADD, .group = InstructionGroup::R_TYPE, .str_label = "ADD", .operand_count = 3}},
    {InstructionLabel::SUB, {.label = InstructionLabel::SUB, .group = InstructionGroup::R_TYPE, .str_label = "SUB", .operand_count = 3}},
    {InstructionLabel::XOR, {.label = InstructionLabel::XOR, .group = InstructionGroup::R_TYPE, .str_label = "XOR", .operand_count = 3}},
    {InstructionLabel::OR, {.label = InstructionLabel::OR, .group = InstructionGroup::R_TYPE, .str_label = "OR", .operand_count = 3}},
    {InstructionLabel::AND, {.label = InstructionLabel::AND, .group = InstructionGroup::R_TYPE, .str_label = "AND", .operand_count = 3}},
    {InstructionLabel::SLL, {.label = InstructionLabel::SLL, .group = InstructionGroup::R_TYPE, .str_label = "SLL", .operand_count = 3}},
    {InstructionLabel::SHR, {.label = InstructionLabel::SHR, .group = InstructionGroup::R_TYPE, .str_label = "SHR", .operand_count = 3}},
    {InstructionLabel::MUL, {.label = InstructionLabel::MUL, .group = InstructionGroup::R_TYPE, .str_label = "MUL", .operand_count = 3}},
    {InstructionLabel::MULH, {.label = InstructionLabel::MULH, .group = InstructionGroup::R_TYPE, .str_label = "MULH", .operand_count = 3}},
    {InstructionLabel::MAC, {.label = InstructionLabel::MAC, .group = InstructionGroup::R_TYPE, .str_label = "MAC", .operand_count = 4}},
    {InstructionLabel::ADDI, {.label = InstructionLabel::ADDI, .group = InstructionGroup::I_TYPE, .str_label = "ADDI", .operand_count = 3}},
    {InstructionLabel::XORI, {.label = InstructionLabel::XORI, .group = InstructionGroup::I_TYPE, .str_label = "XORI", .operand_count = 3}},
    {InstructionLabel::ORI, {.label = InstructionLabel::ORI, .group = InstructionGroup::I_TYPE, .str_label = "ORI", .operand_count = 3}},
    {InstructionLabel::ANDI, {.label = InstructionLabel::ANDI, .group = InstructionGroup::I_TYPE, .str_label = "ANDI", .operand_count = 3}},
    {InstructionLabel::SLLI, {.label = InstructionLabel::SLLI, .group = InstructionGroup::I_TYPE, .str_label = "SLLI", .operand_count = 3}},
    {InstructionLabel::SHRI, {.label = InstructionLabel::SHRI, .group = InstructionGroup::I_TYPE, .str_label = "SHRI", .operand_count = 3}},
    {InstructionLabel::CONVERT, {.label = InstructionLabel::CONVERT, .group = InstructionGroup::C_TYPE, .str_label = "CONVERT", .operand_count = 2}},
};

inline const std::map<InstructionLabel, std::string> LABEL_TO_STRING = {
    // R-type
    {InstructionLabel::ADD, "ADD"},
    {InstructionLabel::SUB, "SUB"},
    {InstructionLabel::XOR, "XOR"},
    {InstructionLabel::OR, "OR"},
    {InstructionLabel::AND, "AND"},
    {InstructionLabel::SLL, "SLL"},
    {InstructionLabel::SHR, "SHR"},
    {InstructionLabel::MUL, "MUL"},
    {InstructionLabel::MULH, "MULH"},
    {InstructionLabel::MAC, "MAC"},
    // I-type
    {InstructionLabel::ADDI, "ADDI"},
    {InstructionLabel::XORI, "XORI"},
    {InstructionLabel::ORI, "ORI"},
    {InstructionLabel::ANDI, "ANDI"},
    {InstructionLabel::SLLI, "SLLI"},
    {InstructionLabel::SHRI, "SHRI"},
    // C-type
    {InstructionLabel::CONVERT, "CONVERT"},
};

inline const std::map<TypeLabels, std::string> LABEL_TO_TYPE_STRING = {
    {TypeLabels::FREE, "free"},
    {TypeLabels::UINT, "uint"},
    {TypeLabels::UMOD, "umod"},
    {TypeLabels::UPACKED, "upacked"},
    {TypeLabels::BOOL, "bool"},
    {TypeLabels::SINT, "sint"},
    {TypeLabels::SMOD, "smod"},
    {TypeLabels::Q32_8, "q32.8"},
    {TypeLabels::Q24_16, "q24.16"},
    {TypeLabels::Q8_32, "q8.32"},
    {TypeLabels::SPACKED, "spacked"},
    {TypeLabels::INSTRUCTION, "instruction"},
    {TypeLabels::PTR, "ptr"},
    {TypeLabels::INERT, "inert"},
};

inline const std::map<std::string, TypeLabels> TYPE_STRING_TO_LABEL = {
    {"free", TypeLabels::FREE},
    {"uint", TypeLabels::UINT},
    {"umod", TypeLabels::UMOD},
    {"upacked", TypeLabels::UPACKED},
    {"bool", TypeLabels::BOOL},
    {"sint", TypeLabels::SINT},
    {"smod", TypeLabels::SMOD},
    {"q32.8", TypeLabels::Q32_8},
    {"q24.16", TypeLabels::Q24_16},
    {"q8.32", TypeLabels::Q8_32},
    {"spacked", TypeLabels::SPACKED},
    {"instruction", TypeLabels::INSTRUCTION},
    {"ptr", TypeLabels::PTR},
    {"inert", TypeLabels::INERT},
};


inline constexpr uint64_t TAG_FIELD_LOW = 40;
inline constexpr uint64_t TAG_FIELD_HIGH = 45;

inline constexpr uint64_t DATA_FIELD_HIGH = 39;
inline constexpr uint64_t DATA_FIELD_LOW = 0;
inline constexpr uint64_t DATA_FIELD_SIZE = 40;

inline constexpr uint64_t OPCODE_FIELD_HIGH = 39;
inline constexpr uint64_t OPCODE_FIELD_LOW = 32;


inline constexpr uint64_t I_TYPE_START_REGISTER_FIELD = 22;
inline constexpr uint64_t I_TYPE_END_REGISTER_FIELD = 31;
inline constexpr uint64_t I_TYPE_REGISTER_FIELD_WIDTH = 5;
inline constexpr uint64_t IMM_FIELD_HIGH = 21;
inline constexpr uint64_t IMM_FIELD_LOW = 0;

inline constexpr uint64_t SATURATION_FLAG = 0;


inline constexpr uint64_t MAX_GPR_RANGE = 22;

struct BVM
{
    size_t PC = 0;
    std::array<uint64_t, 23> GPR{};
    std::array<PointerReg, 13> PR{};
    std::array<uint64_t, 8192> MEM{};
    std::bitset<8> STATUS_REGS;

    BVM()
    {
        GPR[0] |= 1ULL << TAG_FIELD_LOW; // Write UINT to register 0
    }


};


enum class ConvOp : uint8_t {
    Id,       // Identity, no-op
    Retag,    // Change tag only, data bits unchanged
    Zero,     // Clear data bits to 0
    Scale,    // Rescale/bit-shift, saturates on overflow
    Clamp,    // Range/sign check with saturation clamping
    Trunc,    // Truncate fractional bits
    NonZero,  // true if non-zero
    BoolTo,   // false→0, true→1 in target representation
    Invalid,  // Trap
};

using enum ConvOp;

//                          FREE     UINT     UMOD     UPKD     BOOL     SINT     SMOD     Q32_8    Q24_16   Q8_32    SPKD     INSTR    PTR      INERT
inline constexpr std::array<std::array<ConvOp, 14>, 14> CONV_TABLE = {{
/*FREE */  {{ Id,      Zero,    Zero,    Zero,    Zero,    Zero,    Zero,    Zero,    Zero,    Zero,    Zero,    Zero,    Invalid, Zero    }},
/*UINT */  {{ Retag,   Id,      Retag,   Invalid, NonZero, Clamp,   Clamp,   Scale,   Scale,   Scale,   Invalid, Invalid, Invalid, Retag   }},
/*UMOD */  {{ Retag,   Retag,   Id,      Invalid, NonZero, Clamp,   Clamp,   Scale,   Scale,   Scale,   Invalid, Invalid, Invalid, Retag   }},
/*UPKD */  {{ Retag,   Invalid, Invalid, Id,      Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Retag,   Invalid, Invalid, Retag   }},
/*BOOL */  {{ Retag,   BoolTo,  BoolTo,  Invalid, Id,      BoolTo,  BoolTo,  BoolTo,  BoolTo,  BoolTo,  Invalid, Invalid, Invalid, Retag   }},
/*SINT */  {{ Retag,   Clamp,   Clamp,   Invalid, NonZero, Id,      Retag,   Scale,   Scale,   Scale,   Invalid, Invalid, Invalid, Retag   }},
/*SMOD */  {{ Retag,   Clamp,   Clamp,   Invalid, NonZero, Retag,   Id,      Scale,   Scale,   Scale,   Invalid, Invalid, Invalid, Retag   }},
/*Q32_8*/  {{ Retag,   Trunc,   Trunc,   Invalid, NonZero, Trunc,   Trunc,   Id,      Scale,   Scale,   Invalid, Invalid, Invalid, Retag   }},
/*Q24_16*/ {{ Retag,   Trunc,   Trunc,   Invalid, NonZero, Trunc,   Trunc,   Scale,   Id,      Scale,   Invalid, Invalid, Invalid, Retag   }},
/*Q8_32*/  {{ Retag,   Trunc,   Trunc,   Invalid, NonZero, Trunc,   Trunc,   Scale,   Scale,   Id,      Invalid, Invalid, Invalid, Retag   }},
/*SPKD */  {{ Retag,   Invalid, Invalid, Retag,   Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Id,      Invalid, Invalid, Retag   }},
/*INSTR*/  {{ Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Id,      Invalid, Retag   }},
/*PTR  */  {{ Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Invalid, Id,      Invalid }},
/*INERT*/  {{ Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Retag,   Invalid, Id      }},
}};






using uint128_t = __uint128_t;
