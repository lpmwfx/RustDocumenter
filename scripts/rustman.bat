@echo off
:: rustman.bat — open rustdoc-viewer for the nearest man/ directory
::
:: Usage:
::   rustman              walk up from CWD to find man/MANIFEST.json
::   rustman <PATH>       open viewer for given project/man dir
::   rustman gen          generate man/ for CWD first, then open
::   rustman check        check doc coverage for CWD
::
setlocal enabledelayedexpansion

set "CMD=%~1"

:: rustman gen — generate then open
if /i "%CMD%"=="gen" (
    echo Generating man/ ...
    rustdocumenter gen "%CD%"
    if errorlevel 1 exit /b 1
    rustdoc-viewer "%CD%"
    exit /b
)

:: rustman check — coverage check
if /i "%CMD%"=="check" (
    rustdocumenter check "%CD%"
    exit /b
)

:: rustman <explicit path>
if not "%CMD%"=="" (
    rustdoc-viewer "%CMD%"
    exit /b
)

:: No arg — walk up from CWD
set "DIR=%CD%"
:search
if exist "!DIR!\man\MANIFEST.json" (
    rustdoc-viewer "!DIR!"
    exit /b
)
:: Go up one level
for %%I in ("!DIR!") do set "PARENT=%%~dpI"
:: Strip trailing backslash
if "!PARENT:~-1!"=="\" set "PARENT=!PARENT:~0,-1!"
if "!PARENT!"=="!DIR!" (
    echo rustman: no man/ directory found in %CD% or any parent directory
    echo Run "rustman gen" to generate documentation first.
    exit /b 1
)
set "DIR=!PARENT!"
goto search
