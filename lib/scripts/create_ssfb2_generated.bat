@echo off
setlocal
set CURDIR=%~dp0
set BASEDIR=%CURDIR%..
set BUILDDIR=%BASEDIR%\build
@echo on

pushd "%BASEDIR%"

flatc.exe --cpp-std 'c++17' -c "%BASEDIR%\..\utils\fbs\ssfb2.fbs" --gen-compare --gen-object-api --gen-mutable --cpp-ptr-type std::shared_ptr
move /y ssfb2_generated.h include\SSConverter3Lib\ssfb2

popd
