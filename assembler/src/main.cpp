#include "tests.hpp"
#include "assembler.hpp"
#include <cstddef>
#include <cstdio>
#include <exception>
#include <print>
#include <span>

int main(int argc, char *argv[])
{
    run_tests();
    try
    {
        const auto [ASSEMBLY_INPUT, ASSEMBLY_OUTPUT] = process_args(std::span(argv, static_cast<size_t>(argc)));
        const auto input_buffer = clean_input_file(ASSEMBLY_INPUT);
        const auto output_buffer = assemble(input_buffer);
        write_buffer(output_buffer, ASSEMBLY_OUTPUT);
        return 0;
    }
    catch (const std::exception &e)
    {
        std::println(stderr, "Error: {}", e.what());
        return 1;
    }
}
