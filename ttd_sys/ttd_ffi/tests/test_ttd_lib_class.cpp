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

struct ReplayEngineTestClass
{
    TTD_FFI::Replay::ReplayEngine Engine;
    ReplayEngineTestClass() : Engine()
    {
        const auto valid_path = GetTemp() / L"test.run";
        this->Engine.Load((const u16*)valid_path.c_str());
    }
};


TEST_CASE_METHOD(ReplayEngineTestClass, "Basic Test", "Navigate trace")
{
    auto RawCursor = this->Engine.NewCursor();
    REQUIRE(RawCursor != 0);

    auto Cursor = TTD_FFI::Replay::ReplayCursor(RawCursor);
    REQUIRE(Cursor.GetPosition() == TTD::Replay::Position::Invalid);

    // Move cursor positions
    for ( auto i = 0; i < 10; i++ )
    {
        Cursor.SetPosition(Engine.GetLifetime().Min);
        REQUIRE(Cursor.GetPosition() == Engine.GetLifetime().Min);

        Cursor.SetPosition(Engine.GetLifetime().Max);
        REQUIRE(Cursor.GetPosition() == Engine.GetLifetime().Max);
    }

    auto const& Lifetime = Engine.GetLifetime();

    // Replay forward/backward
    for ( auto i = 0; i < 10; i++ )
    {
        TTD::Replay::ICursorView::ReplayResult fwd_res {};
        Cursor.ReplayBackward(Lifetime.Min, &fwd_res);
        REQUIRE(Cursor.GetPosition() == Engine.GetLifetime().Min);

        TTD::Replay::ICursorView::ReplayResult bkw_res {};
        Cursor.ReplayForward(Lifetime.Max, &bkw_res);
        REQUIRE(Cursor.GetPosition() == Engine.GetLifetime().Max);
    }
}
