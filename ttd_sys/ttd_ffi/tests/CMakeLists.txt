#
# Unit tests
#
project(ttd_ffi_tests)

set(PROJECT_TEST_DIR ${PROJECT_ROOT_DIR}/tests CACHE INTERNAL "PROJECT_TEST_DIR")

include(CTest)
find_package(Catch2 REQUIRED)
enable_testing()

set(TTD_SDK_NAME microsoft.timetraveldebugging.apis)
set(TTD_SDK_VERSION 0.9.5)
set(TTD_SDK_PATH $ENV{USERPROFILE}/.nuget/packages/${TTD_SDK_NAME}/${TTD_SDK_VERSION})
list(APPEND CMAKE_MODULE_PATH "${TTD_SDK_PATH}/cmake")
include(Microsoft.TimeTravelDebugging.ApisConfig)


set(COMMON_TEST_DIR ${PROJECT_TEST_DIR})
set(
    SOURCE_FILES

    ${COMMON_TEST_DIR}/test_ttd_lib_basic.cpp
    ${COMMON_TEST_DIR}/test_ttd_lib_class.cpp
)

add_compile_definitions(_UNICODE UNICODE)

add_executable(${PROJECT_NAME} ${SOURCE_FILES})
add_dependencies(${PROJECT_NAME} ${CMAKE_PROJECT_NAME})


add_test(
    NAME ${PROJECT_NAME}
    COMMAND $<TARGET_FILE:${PROJECT_NAME}>
)

set_target_properties(
    ${PROJECT_NAME}
    PROPERTIES
    CXX_STANDARD 23
)


add_dependencies(${PROJECT_NAME} ${CMAKE_PROJECT_NAME} Catch2)
target_compile_features(${PROJECT_NAME} PRIVATE cxx_std_23)
target_include_directories(${PROJECT_NAME}
    PRIVATE
        $<TARGET_PROPERTY:${CMAKE_PROJECT_NAME},INCLUDE_DIRECTORIES>
        $<TARGET_PROPERTY:${CMAKE_PROJECT_NAME},SOURCE_DIR>
        Catch2::Catch2WithMain
)
target_link_libraries(${PROJECT_NAME}
    PUBLIC
        $<TARGET_LINKER_FILE:${CMAKE_PROJECT_NAME}>
        $<TARGET_PROPERTY:${CMAKE_PROJECT_NAME},LINK_INTERFACE_LIBRARIES>
        $<TARGET_PROPERTY:${CMAKE_PROJECT_NAME},LINK_LIBRARIES>
        Catch2::Catch2WithMain
)
target_link_options(${PROJECT_NAME}
    PUBLIC

    # Link ASAN for debug
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},win32>:$<$<CONFIG:Debug>:/InferAsanLibs>>
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},x64>:$<$<CONFIG:Debug>:/InferAsanLibs>>

    # Fix issues with SAFESEH linking for 32b
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},win32>:/SAFESEH:NO>
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},arm>:/SAFESEH:NO>
)

add_custom_command(
    TARGET ${PROJECT_NAME} POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        $<TARGET_FILE:${CMAKE_PROJECT_NAME}> $<TARGET_FILE_DIR:${PROJECT_NAME}>
)

install(TARGETS ${PROJECT_NAME} DESTINATION ${PROJECT_NAME}/Tests )
