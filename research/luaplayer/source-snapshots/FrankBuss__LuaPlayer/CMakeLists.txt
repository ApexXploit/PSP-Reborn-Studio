cmake_minimum_required(VERSION 3.10)
project(LuaPlayer VERSION 0.20 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_C_STANDARD 99)

# Build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Compiler flags
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall")
set(CMAKE_CXX_FLAGS_DEBUG "-g -O0")
set(CMAKE_CXX_FLAGS_RELEASE "-O2")

# Find required packages
find_package(Freetype REQUIRED)
find_package(PNG REQUIRED)
find_package(JPEG REQUIRED)
find_package(Threads REQUIRED)

# Find Lua 5.4+
find_package(Lua 5.4 REQUIRED)

# Find SDL2
find_package(PkgConfig REQUIRED)
pkg_check_modules(SDL2 REQUIRED sdl2)

# Find MikMod
pkg_check_modules(MIKMOD QUIET libmikmod)
if(NOT MIKMOD_FOUND)
    find_path(MIKMOD_INCLUDE_DIRS mikmod.h
        HINTS /usr/include /usr/local/include)
    find_library(MIKMOD_LIBRARIES mikmod
        HINTS /usr/lib /usr/local/lib /usr/lib/x86_64-linux-gnu)
    if(MIKMOD_INCLUDE_DIRS AND MIKMOD_LIBRARIES)
        set(MIKMOD_FOUND TRUE)
    endif()
endif()
if(NOT MIKMOD_FOUND)
    message(FATAL_ERROR "MikMod not found. Please install libmikmod-dev")
endif()

# Function to convert binary file to C array
function(bin2c INPUT_FILE OUTPUT_FILE ARRAY_NAME)
    file(READ ${INPUT_FILE} HEX_CONTENT HEX)
    string(LENGTH "${HEX_CONTENT}" HEX_LEN)
    math(EXPR BYTE_LEN "${HEX_LEN} / 2")

    # Convert hex to comma-separated bytes
    string(REGEX REPLACE "([0-9a-f][0-9a-f])" "0x\\1," HEX_CONTENT "${HEX_CONTENT}")

    # Write the C file
    file(WRITE ${OUTPUT_FILE}
        "unsigned char ${ARRAY_NAME}[] = {\n${HEX_CONTENT}\n};\nunsigned int size_${ARRAY_NAME} = ${BYTE_LEN};\n")
endfunction()

# Generate font files from TTF
bin2c(${CMAKE_SOURCE_DIR}/src/auxiliary/Vera.ttf ${CMAKE_BINARY_DIR}/vera.cpp ttfVera)
bin2c(${CMAKE_SOURCE_DIR}/src/auxiliary/VeraMono.ttf ${CMAKE_BINARY_DIR}/veraMono.cpp ttfVeraMono)

# Source files - core LuaPlayer
set(LUAPLAYER_SOURCES
    src/graphics.cpp
    src/sound.cpp
    src/luaplayer.cpp
    src/luacontrols.cpp
    src/luagraphics.cpp
    src/luasound.cpp
    src/luasystem.cpp
    src/luatimer.cpp
    src/lua3d.cpp
    src/utility.cpp
)

# Source files - Linux platform
set(PLATFORM_SOURCES
    src/platform/platform_linux.cpp
    src/platform/psp_stubs.cpp
    src/platform/md5.cpp
)

# Create executable
add_executable(luaplayer
    ${LUAPLAYER_SOURCES}
    ${PLATFORM_SOURCES}
)

# Define PLATFORM_LINUX for conditional compilation
target_compile_definitions(luaplayer PRIVATE PLATFORM_LINUX)

# Include directories
target_include_directories(luaplayer PRIVATE
    ${CMAKE_SOURCE_DIR}/src
    ${CMAKE_SOURCE_DIR}/src/platform
    ${CMAKE_BINARY_DIR}
    ${FREETYPE_INCLUDE_DIRS}
    ${PNG_INCLUDE_DIRS}
    ${JPEG_INCLUDE_DIR}
    ${LUA_INCLUDE_DIR}
    ${LUA_INCLUDE_DIRS}
    ${SDL2_INCLUDE_DIRS}
    ${MIKMOD_INCLUDE_DIRS}
)

# Link libraries
target_link_libraries(luaplayer
    ${FREETYPE_LIBRARIES}
    ${PNG_LIBRARIES}
    ${JPEG_LIBRARIES}
    ${LUA_LIBRARIES}
    ${SDL2_LIBRARIES}
    ${MIKMOD_LIBRARIES}
    Threads::Threads
    m
)

# Installation
install(TARGETS luaplayer DESTINATION bin)

# Print configuration summary
message(STATUS "")
message(STATUS "LuaPlayer build configuration:")
message(STATUS "  Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  FreeType: ${FREETYPE_LIBRARIES}")
message(STATUS "  PNG: ${PNG_LIBRARIES}")
message(STATUS "  JPEG: ${JPEG_LIBRARIES}")
message(STATUS "  Lua: ${LUA_LIBRARIES}")
message(STATUS "  SDL2: ${SDL2_LIBRARIES}")
message(STATUS "  MikMod: ${MIKMOD_LIBRARIES}")
message(STATUS "")
