@echo off
setlocal
cd /d "%~dp0.."
title QLCV - Build QLCV

echo ========================================================
echo     DONG GOI PHAN MEM QLCV DESKTOP
echo ========================================================
echo.

rem 1. Kiem tra Node.js / npm
where npm >nul 2>&1
if errorlevel 1 (
    echo [LOI] Khong tim thay npm. Hay cai Node.js.
    pause
    exit /b 1
)

rem 2. Kiem tra Rust / Cargo
where cargo >nul 2>&1
if errorlevel 1 (
    echo [LOI] Khong tim thay Cargo. Hay cai Rust.
    pause
    exit /b 1
)

rem 3. Kiem tra package.json
if not exist "package.json" (
    echo [LOI] Khong tim thay package.json.
    pause
    exit /b 1
)

rem 4. Cai dependencies
if not exist "node_modules\" (
    echo [INFO] Dang cai dependencies...
    call npm ci
    if errorlevel 1 (
        echo [LOI] npm ci that bai.
        pause
        exit /b 1
    )
)

rem 5. Build Tauri
echo.
echo [INFO] Dang build Tauri Release...

call npm run tauri build -- --no-sign
if errorlevel 1 (
    echo [LOI] Build Tauri that bai.
    pause
    exit /b 1
)

rem 6. Tao thu muc dist
if not exist "dist\" mkdir "dist"
if errorlevel 1 (
    echo [LOI] Khong tao duoc thu muc dist.
    pause
    exit /b 1
)

rem 7. Kiem tra va copy executable
rem Luu y: xac nhan ten executable trong Cargo.toml
set "APP_EXE=src-tauri\target\release\tauri-app.exe"

if not exist "%APP_EXE%" (
    echo [LOI] Khong tim thay executable: %APP_EXE%
    echo Hay kiem tra ten binary trong src-tauri\Cargo.toml.
    pause
    exit /b 1
)

copy /y "%APP_EXE%" "dist\QLCV.exe" >nul
if errorlevel 1 (
    echo [LOI] Sao chep QLCV.exe that bai.
    pause
    exit /b 1
)

rem 8. Copy NSIS installer neu co
set "NSIS=src-tauri\target\release\bundle\nsis\QLCV_1.0.0_x64-setup.exe"

if exist "%NSIS%" (
    copy /y "%NSIS%" "dist\QLCV_NSIS_Setup.exe" >nul
    if errorlevel 1 (
        echo [LOI] Sao chep NSIS installer that bai.
        pause
        exit /b 1
    )
) else (
    echo [CANH BAO] Khong tim thay NSIS installer.
)

rem 9. Copy MSI neu co
set "MSI=src-tauri\target\release\bundle\msi\QLCV_1.0.0_x64_en-US.msi"

if exist "%MSI%" (
    copy /y "%MSI%" "dist\QLCV_1.0.0_x64_en-US.msi" >nul
    if errorlevel 1 (
        echo [LOI] Sao chep MSI that bai.
        pause
        exit /b 1
    )
) else (
    echo [CANH BAO] Khong tim thay MSI.
)

rem KHONG copy .env vao dist.
rem Khong dong goi MongoDB URI / API key / SMTP password.

rem 10. Build Inno Setup neu co
set "ISCC_CMD="

if exist "%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" (
    set "ISCC_CMD=%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe"
)

if not defined ISCC_CMD if exist "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" (
    set "ISCC_CMD=C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
)

if not defined ISCC_CMD if exist "C:\Program Files\Inno Setup 6\ISCC.exe" (
    set "ISCC_CMD=C:\Program Files\Inno Setup 6\ISCC.exe"
)

if defined ISCC_CMD (
    if exist "installer.iss" (
        echo.
        echo [INFO] Dang build Inno Setup...
        "%ISCC_CMD%" "installer.iss"
        if errorlevel 1 (
            echo [LOI] Inno Setup build that bai.
            pause
            exit /b 1
        )
    ) else (
        echo [CANH BAO] Khong tim thay installer.iss.
    )
) else (
    echo [CANH BAO] Khong tim thay Inno Setup Compiler.
)

rem 11. Kiem tra ket qua
echo.
echo ========================================================
echo                 KET QUA DONG GOI
echo ========================================================
echo.

if not exist "dist\QLCV.exe" (
    echo [LOI] Thieu QLCV.exe.
    pause
    exit /b 1
)

dir "dist\*.exe" "dist\*.msi" 2>nul

echo.
echo [INFO] Da hoan tat cac buoc build va kiem tra.
echo [INFO] Kiem tra danh sach file phia tren truoc khi phat hanh.
echo.
pause
endlocal
