FLAGS := --Mdir .vbuild --timescale 1ns/1ps --trace-fst -x-assign unique --binary -Wall --trace-structs --trace-max-array 0 --quiet-build -j 0 -CFLAGS -O0
SRC := $(wildcard *.sv)
TOP := tb_and_gate
 


build:
	mkdir -p .vbuild && verilator $(FLAGS) --top-module $(TOP) $(SRC) 
run: build
	./.vbuild/V$(TOP) +verilator+rand+reset+2
wave: run
	surfer -s state.ron wave.fst
lint: 
	verilator --lint-only --top-module $(TOP) $(SRC)  
clean:
	rm -rf .vbuild/* wave.fst
