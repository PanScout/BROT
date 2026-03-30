#pragma once
#include "assembler.hpp"
#include "typedefs.hpp"
#include <bitset>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <format>
#include <print>
#include <string>
#include <utility>
#include <vector>

std::string assm(const std::string &s)
{
    auto to_bin_string = [](const std::string &hex) -> std::string
    {
        uint64_t ret = std::stoul(hex, nullptr, 16);
        return std::bitset<46>(ret).to_string();
    };

    std::vector<std::pair<string, size_t>> input;
    input.emplace_back(s, 1);
    auto ret = assemble(input);
    return to_bin_string(ret.front());
}

struct R_Type
{
    std::string tag = "001011";
    std::string opcode;
    std::string rd, rs1, rs2;
    std::string padding = "00000000000000000";
    constexpr R_Type(InstructionLabel opcode, size_t rd, size_t rs1, size_t rs2)
    {
 
        this->opcode = std::bitset<8>(std::to_underlying(opcode)).to_string();
        this->rd = std::bitset<5>(rd).to_string();
        this->rs1 = std::bitset<5>(rs1).to_string();
        this->rs2 = std::bitset<5>(rs2).to_string();
    }
    [[nodiscard]] constexpr std::string to_str() const
    {
        return tag + opcode + rd + rs1 + rs2 + padding;
    }
};

struct I_Type
{
    std::string tag = "001011";
    std::string opcode;
    std::string rd, rs1;
    std::string imm;
    constexpr I_Type(InstructionLabel opcode, size_t rd, size_t rs1, string imm) : imm(std::move(imm))
    {
 
        this->opcode = std::bitset<8>(std::to_underlying(opcode)).to_string();
        this->rd = std::bitset<5>(rd).to_string();
        this->rs1 = std::bitset<5>(rs1).to_string();
    }
    [[nodiscard]] constexpr std::string to_str() const
    {
        string ret = tag + opcode + rd + rs1 + imm;
        assert (ret.size() == 46);
        return  ret;
    }
};

bool diff_strings(const string &A, const string &B)
{
    bool is_diff = false;
    for(size_t i = 0; i < 46; i++)
    {
        if(A.at(i) != B.at(i))
            is_diff = true;
    }

    if(is_diff)
    {
        std::println("{:<50}{:46}", "String Excepted: " ,A);
        std::println("{:<50}{:46}", "String Got:", B);
    }

    return is_diff;
}


void run_tests()
{
    //R-Type
    assert(assm("ADD r3, r2, r1") == R_Type(InstructionLabel::ADD, 3, 2, 1).to_str());
    assert(assm("ADD r1, r2, r3") == R_Type(InstructionLabel::ADD, 1, 2, 3).to_str());
    assert(assm("SLL r1, r2, r3") == R_Type(InstructionLabel::SLL, 1, 2, 3).to_str());
    assert(assm("MAC r1, r2, r3") == R_Type(InstructionLabel::MAC, 1, 2, 3).to_str());

    //I-type
    //Normal I
    assert(diff_strings(I_Type(InstructionLabel::ADDI, 3, 2, "0000000000001000110100").to_str(), assm("ADDI r3, r2, 564:uint")) == false);
    std::println("TESTS PASSED");
}
