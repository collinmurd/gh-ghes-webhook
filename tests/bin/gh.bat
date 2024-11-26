@echo off
REM this is a mock for the gh cli

REM if the args are ["auth", "token"], then return a fake token
if "%1"=="auth" if "%2"=="token" (
    echo gh_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    exit /b 0
)

REM Default exit
exit /b 1