project(ttd_ffi
        LANGUAGES CXX
        VERSION 0.1.0)

set(PROJECT_LIB_SRC_DIR ${PROJECT_ROOT_DIR}/src CACHE INTERNAL "PROJECT_LIB_SRC_DIR")

#
# Create the constants.hpp macro file
#
configure_file(
    "${PROJECT_LIB_SRC_DIR}/constants.hpp.in"
    "${PROJECT_LIB_SRC_DIR}/constants.hpp"
    NEWLINE_STYLE WIN32
)

#
# TTD SDK API deps
#
set(TTD_SDK_NAME microsoft.timetraveldebugging.apis)
set(TTD_SDK_VERSION 0.9.5)
set(TTD_SDK_PATH $ENV{USERPROFILE}/.nuget/packages/${TTD_SDK_NAME}/${TTD_SDK_VERSION})
list(APPEND CMAKE_MODULE_PATH "${TTD_SDK_PATH}/cmake")
include(Microsoft.TimeTravelDebugging.ApisConfig)

#
# Print out cmake compile info
#
message(STATUS "CMAKE_MODULE_PATH:                 ${CMAKE_MODULE_PATH}")
message(STATUS "CMAKE_SYSTEM_PROCESSOR:            ${CMAKE_SYSTEM_PROCESSOR}")
message(STATUS "CMAKE_SYSTEM_NAME:                 ${CMAKE_SYSTEM_NAME}")

add_library(${PROJECT_NAME} STATIC
    "${PROJECT_LIB_SRC_DIR}/ttd_ffi.cpp"
)

target_include_directories(${PROJECT_NAME}
    PUBLIC
        Microsoft.TimeTravelDebugging.Apis::Includes
)

target_link_libraries(${PROJECT_NAME}
    PUBLIC
        Microsoft.TimeTravelDebugging.Apis::TTDReplay
        Microsoft.TimeTravelDebugging.Apis::TTDLiveRecorder
)

set_target_properties(
    ${PROJECT_NAME}
    PROPERTIES
        CXX_STANDARD 23
        # wtf - doesn't work ???
        # VS_PACKAGE_REFERENCES "Microsoft.TimeTravelDebugging.Apis_0.9.5"
)

target_compile_features(${PROJECT_NAME} PRIVATE cxx_std_23)

get_target_property( TTDSdkHeaderDirs Microsoft.TimeTravelDebugging.Apis::Includes INTERFACE_INCLUDE_DIRECTORIES)
get_target_property( TTDReplayLibs Microsoft.TimeTravelDebugging.Apis::TTDReplay IMPORTED_LOCATION)
get_target_property( TTDLiveRecorderLibs Microsoft.TimeTravelDebugging.Apis::TTDLiveRecorder IMPORTED_LOCATION)

# Compile and Link ASAN for debug
target_compile_options(${PROJECT_NAME}
    PUBLIC

    # ASAN only supported for Intel
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},x64>:$<$<CONFIG:Debug>:/fsanitize=address>>
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},win32>:$<$<CONFIG:Debug>:/fsanitize=address>>
)

target_link_options(${PROJECT_NAME}
    PUBLIC

    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},win32>:$<$<CONFIG:Debug>:/InferAsanLibs>>
    $<$<STREQUAL:${CMAKE_GENERATOR_PLATFORM},x64>:$<$<CONFIG:Debug>:/InferAsanLibs>>
)

install(TARGETS ${PROJECT_NAME} DESTINATION ${PROJECT_NAME}/Library )
install(
    FILES
        ${TTDReplayLibs}
        ${TTDLiveRecorderLibs}
    DESTINATION ${PROJECT_NAME}/Library)
install(
    FILES
        "${PROJECT_LIB_SRC_DIR}/constants.hpp"
        "${PROJECT_LIB_SRC_DIR}/ttd_ffi.hpp"
    DESTINATION ${PROJECT_NAME}/Include)
install(
    DIRECTORY ${TTDSdkHeaderDirs}/
    DESTINATION ${PROJECT_NAME}/Include
    FILES_MATCHING PATTERN "*.h")
