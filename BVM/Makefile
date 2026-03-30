all: compile

compile:
	cmake -B build -G Ninja -S . -DCMAKE_TOOLCHAIN_FILE=$$VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake -DCMAKE_CXX_COMPILER=clang++
	cmake --build build

release:
	cmake -B build-release -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_COMPILER=clang++ 
	cmake --build build-release

clean:
	rm -rf build build-release bin/*

lint:


.PHONY: compile release clean
