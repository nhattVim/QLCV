@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0.."
title QLCV - Quan ly Phat hanh va Ban cap nhat

:MENU
cls
set "CHOICE="
echo =====================================================================
echo           HE THONG PHAT HANH BAN CAP NHAT QLCV (TAURI v2)
echo =====================================================================
echo.
echo   [1] Phat hanh ban moi (Nang version, tao tag, chay GitHub Actions)
echo   [2] Xoa ban phat hanh / Git Tag (Xoa tag tren may va tren GitHub)
echo   [3] Thoat
echo.
echo =====================================================================
set /p CHOICE="Vui long chon [1-3]: "

if "!CHOICE!"=="1" goto RELEASE
if "!CHOICE!"=="2" goto DELETE_RELEASE
if "!CHOICE!"=="3" goto EXIT
goto MENU

:RELEASE
cls
set "INPUT_VER="
set "CONFIRM="
echo =====================================================================
echo                    [1] PHAT HANH PHIEN BAN MOI
echo =====================================================================
echo.
echo Nhap so phien ban muon phat hanh (Vi du: 1.0.1 hoac v1.0.1):
set /p INPUT_VER="> Version: "

if "%INPUT_VER%"=="" (
    echo [Loi] Ban chua nhap phien ban.
    pause
    goto MENU
)

set "CLEAN_VER=%INPUT_VER%"
if "%CLEAN_VER:~0,1%"=="v" set "CLEAN_VER=%CLEAN_VER:~1%"
if "%CLEAN_VER:~0,1%"=="V" set "CLEAN_VER=%CLEAN_VER:~1%"
set "TAG_NAME=v%CLEAN_VER%"

echo.
echo Phien ban duoc chon: %CLEAN_VER% (Git Tag: %TAG_NAME%)
echo.
set /p CONFIRM="Ban co chac chan muon phat hanh %TAG_NAME%? (y/n): "
if /i not "%CONFIRM%"=="y" goto MENU

echo.
echo [1/5] Dang cap nhat version trong package.json va tauri.conf.json...
node -e "const fs=require('fs'); ['package.json','src-tauri/tauri.conf.json'].forEach(p => { if (fs.existsSync(p)) { const j=JSON.parse(fs.readFileSync(p,'utf8')); j.version='%CLEAN_VER%'; fs.writeFileSync(p, JSON.stringify(j, null, 2)+'\n','utf8'); } });"

echo [2/5] Dang commit toan bo ma nguon va phien ban moi...
git add -A
git commit -m "release: %TAG_NAME%" >nul 2>&1

echo [3/5] Dang day code len nhanh main...
git push origin main
if %ERRORLEVEL% neq 0 (
    echo [Canh bao] Khong the push len nhanh main. Vui long kiem tra ket noi Git.
)

echo [4/5] Dang tao va day Git Tag %TAG_NAME% len GitHub...
git tag -d %TAG_NAME% >nul 2>&1
git tag -a %TAG_NAME% -m "Release %TAG_NAME%"
git push origin %TAG_NAME%
if %ERRORLEVEL% neq 0 (
    echo [Loi] Khong the push tag %TAG_NAME% len GitHub!
    pause
    goto MENU
)

echo.
echo =====================================================================
echo                      PHAT HANH THANH CONG!
echo =====================================================================
echo Da day Git Tag %TAG_NAME% len GitHub de kich hoat GitHub Actions!
echo.
echo * Cac buoc theo doi tiep theo:
echo   1. Xem tien trinh build tu dong:
echo      Link: https://github.com/nhattVim/QLCV/actions
echo.
echo   2. Khi build xong (khoang 3-5 phut), file cai dat se xuat hien tai:
echo      Link: https://github.com/nhattVim/QLCV/releases/tag/%TAG_NAME%
echo.
echo   3. Tat ca cac may da cai app QLCV khi mo len se tu dong hien
echo      nut thong bao cap nhat len ban %TAG_NAME% tren thanh tieu de!
echo =====================================================================
echo.
pause
goto MENU

:DELETE_RELEASE
cls
echo =====================================================================
echo                 [2] XOA BAN PHAT HANH / GIT TAG
echo =====================================================================
echo.
echo Dang kiem tra va dong bo Git Tag tu GitHub...
git fetch --tags --prune-tags origin >nul 2>&1

set "TAG_COUNT=0"
for /f "tokens=*" %%T in ('git tag -l') do (
    set /a TAG_COUNT+=1
    set "TAG_!TAG_COUNT!=%%T"
)

if !TAG_COUNT! equ 0 (
    echo.
    echo [Thong bao] Khong tim thay Git Tag nao tren may va GitHub!
    echo.
    pause
    goto MENU
)

echo.
echo Danh sach Git Tag hien co:
echo ---------------------------------------------------------------------
call :PRINT_TAGS !TAG_COUNT!
echo ---------------------------------------------------------------------
echo   [0]   Quay lai Menu chinh
echo   [ALL] Xoa tat ca cac tag tren
echo =====================================================================
echo.
set "SEL="
set /p SEL="Nhap cac so thu tu muon xoa (vi du: 1 3 hoac 1,2 hoac ALL): "

if "!SEL!"=="" goto MENU
if "!SEL!"=="0" goto MENU

set "SELECTED_TAGS="
if /i "!SEL!"=="all" (
    call :SELECT_ALL_TAGS !TAG_COUNT!
) else (
    set "NORMALIZED_SEL=!SEL:,= !"
    for %%N in (!NORMALIZED_SEL!) do (
        if defined TAG_%%N (
            set "SELECTED_TAGS=!SELECTED_TAGS! !TAG_%%N!"
        )
    )
)

if "!SELECTED_TAGS!"=="" (
    echo.
    echo [Loi] Khong co lua chon hop le nao!
    echo.
    pause
    goto MENU
)

echo.
echo CANH BAO: Cac Git Tag sau day se bi xoa vinh vien tren MAY va GITHUB:
for %%T in (!SELECTED_TAGS!) do (
    echo   - %%T
)
echo.
set "CONFIRM_DEL="
set /p CONFIRM_DEL="Ban co chac chan 100%% muon xoa cac tag tren? (y/n): "
if /i not "!CONFIRM_DEL!"=="y" goto MENU

set "HAS_GH=0"
where gh >nul 2>&1 && set "HAS_GH=1"

echo.
echo Dang tien hanh xoa...
for %%T in (!SELECTED_TAGS!) do (
    echo.
    echo -- Dang xoa tag: %%T
    git tag -d %%T >nul 2>&1
    git push origin --delete %%T
    if "!HAS_GH!"=="1" (
        gh release delete %%T --yes --cleanup-tag >nul 2>&1
    )
)

echo.
echo =====================================================================
echo                 DA XOA HOAN TAT CAC TAG DA CHON!
echo =====================================================================
echo.
pause
goto MENU

:PRINT_TAGS
for /l %%I in (1,1,%1) do (
    echo   [%%I] !TAG_%%I!
)
exit /b

:SELECT_ALL_TAGS
for /l %%I in (1,1,%1) do (
    set "SELECTED_TAGS=!SELECTED_TAGS! !TAG_%%I!"
)
exit /b

:EXIT
exit /b 0
