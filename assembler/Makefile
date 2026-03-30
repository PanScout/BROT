all: compile

compile:
	cmake -B build -G Ninja -DCMAKE_CXX_COMPILER=clang++
	cmake --build build

release:
	cmake -B build-release -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_COMPILER=clang++
	cmake --build build-release

run:
	./bin/basm

clean:
	rm -rf build build-release bin/*

.PHONY: compile release run clean
