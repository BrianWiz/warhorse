cmake_minimum_required(VERSION 3.10)
project(RustCppIntegration)

set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug")

if(WIN32)
    set(RUST_LIB "${RUST_LIB_PATH}/warhorse_cpp.dll.lib")
    set(RUST_DLL "${RUST_LIB_PATH}/warhorse_cpp.dll")
else()
    set(RUST_LIB "${RUST_LIB_PATH}/libwarhorse_cpp.so")
endif()

add_executable(warhorse_cpp_client src/client.cpp)
target_link_libraries(warhorse_cpp_client ${RUST_LIB})

# Copy DLL to output directory
if(WIN32)
    add_custom_command(TARGET warhorse_cpp_client POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${RUST_DLL}"
        $<TARGET_FILE_DIR:warhorse_cpp_client>)
endif()
