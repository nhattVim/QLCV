@echo off
setlocal enabledelayedexpansion
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

if "%CHOICE%"=="1" goto RELEASE
if "%CHOICE%"=="2" goto DELETE_RELEASE
if "%CHOICE%"=="3" goto EXIT
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
set "DEL_VER="
set "CONFIRM_DEL="
echo =====================================================================
echo                 [2] XOA BAN PHAT HANH / GIT TAG
echo =====================================================================
echo.
echo Nhap so phien ban / tag muon xoa (Vi du: 1.0.1 hoac v1.0.1):
set /p DEL_VER="> Version / Tag: "

if "%DEL_VER%"=="" (
    echo [Loi] Ban chua nhap tag can xoa.
    pause
    goto MENU
)

set "CLEAN_DEL=%DEL_VER%"
if "%CLEAN_DEL:~0,1%"=="v" set "CLEAN_DEL=%CLEAN_DEL:~1%"
if "%CLEAN_DEL:~0,1%"=="V" set "CLEAN_DEL=%CLEAN_DEL:~1%"
set "DEL_TAG=v%CLEAN_DEL%"

echo.
echo CANH BAO: Thao tac nay se xoa tag %DEL_TAG% o ca may nay va tren GitHub!
set /p CONFIRM_DEL="Ban co chac chan 100%% muon xoa %DEL_TAG%? (y/n): "
if /i not "%CONFIRM_DEL%"=="y" goto MENU

echo.
echo [1/3] Dang xoa local tag (%DEL_TAG% va %CLEAN_DEL%)...
git tag -d %DEL_TAG% >nul 2>&1
git tag -d %CLEAN_DEL% >nul 2>&1

echo [2/3] Dang xoa remote tag tren GitHub...
git push origin --delete %DEL_TAG% >nul 2>&1
git push origin --delete %CLEAN_DEL% >nul 2>&1

echo [3/3] Dang kiem tra xoa GitHub Release...
where gh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    gh release delete %DEL_TAG% --yes --cleanup-tag >nul 2>&1
    gh release delete %CLEAN_DEL% --yes --cleanup-tag >nul 2>&1
    echo [OK] Da xoa ban Release tren GitHub thong qua GitHub CLI.
) else (
    echo.
    echo [Luu y] De xoa han ban Release tren web GitHub (neu chua xoa):
    echo   Link 1: https://github.com/nhattVim/QLCV/releases/tag/%DEL_TAG%
    echo   Link 2: https://github.com/nhattVim/QLCV/releases/tag/%CLEAN_DEL%
    echo Bam vao nut 'Delete release' de xoa.
)

echo.
echo =====================================================================
echo                    DA XOA HOAN TAT TAG %DEL_TAG%!
echo =====================================================================
echo.
pause
goto MENU

:EXIT
exit /b 0
