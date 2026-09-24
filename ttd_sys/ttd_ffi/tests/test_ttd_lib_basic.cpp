#define CATCH_CONFIG_MAIN

#include <windows.h>

#include <catch2/catch_test_macros.hpp>
#include <filesystem>

#define private public
#define protected public

#include "ttd_ffi.hpp"

#define NS "Basic"

static std::filesystem::path
GetTemp()
{
    wchar_t tempPath[MAX_PATH] {};
    ::GetTempPathW(MAX_PATH, tempPath);
    return std::filesystem::path(tempPath);
}

TEST_CASE("TTD FFI Tests", "[" NS "]")
{
    SECTION("Load trace")
    {
        auto Engine = TTD_FFI::Replay::ReplayEngine();

        const auto invalid_path = GetTemp() / L"iDontExist";
        REQUIRE(Engine.Load((const u16*)invalid_path.c_str()) == ERROR_NOT_FOUND);

        const auto valid_path = GetTemp() / L"test.run";
        REQUIRE(Engine.Load((const u16*)valid_path.c_str()) == ERROR_SUCCESS);
    }
}
