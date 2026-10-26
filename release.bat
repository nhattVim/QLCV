@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
title QLCV - Quản lý Phát hành & Bản cập nhật

:MENU
cls
echo =====================================================================
echo           HỆ THỐNG PHÁT HÀNH BẢN CẬP NHẬT QLCV (TAURI v2)
echo =====================================================================
echo.
echo   [1] Phát hành bản mới (Tự động nâng version, tạo tag, kích hoạt GitHub Actions)
echo   [2] Xóa bản phát hành / tag (Xóa local tag và remote tag trên GitHub)
echo   [3] Thoát
echo.
echo =====================================================================
set /p CHOICE="Vui lòng chọn [1-3]: "

if "%CHOICE%"=="1" goto RELEASE
if "%CHOICE%"=="2" goto DELETE_RELEASE
if "%CHOICE%"=="3" goto EXIT
goto MENU

:RELEASE
cls
echo =====================================================================
echo                    [1] PHÁT HÀNH PHIÊN BẢN MỚI
echo =====================================================================
echo.
echo Nhập số phiên bản muốn phát hành (Ví dụ: 1.0.1 hoặc v1.0.1):
set /p INPUT_VER="> Version: "

if "%INPUT_VER%"=="" (
    echo [Lỗi] Bạn chưa nhập phiên bản.
    pause
    goto MENU
)

:: Chuẩn hóa version (bỏ chữ v nếu có)
set "CLEAN_VER=%INPUT_VER%"
if "%CLEAN_VER:~0,1%"=="v" set "CLEAN_VER=%CLEAN_VER:~1%"
if "%CLEAN_VER:~0,1%"=="V" set "CLEAN_VER=%CLEAN_VER:~1%"
set "TAG_NAME=v%CLEAN_VER%"

echo.
echo Phiên bản được chọn: %CLEAN_VER% (Git Tag: %TAG_NAME%)
echo.
set /p CONFIRM="Bạn có chắc chắn muốn phát hành %TAG_NAME%? (y/n): "
if /i not "%CONFIRM%"=="y" goto MENU

echo.
echo [1/5] Đang cập nhật version trong package.json và tauri.conf.json...
powershell -NoProfile -Command "(Get-Content package.json) -replace '\"version\":\s*\"[^\"]+\"', '\"version\": \"%CLEAN_VER%\"' | Set-Content package.json"
powershell -NoProfile -Command "(Get-Content src-tauri/tauri.conf.json) -replace '\"version\":\s*\"[^\"]+\"', '\"version\": \"%CLEAN_VER%\"' | Set-Content src-tauri/tauri.conf.json"

echo [2/5] Đang commit phiên bản mới...
git add package.json src-tauri/tauri.conf.json
git commit -m "release: bump version to %TAG_NAME%" >nul 2>&1

echo [3/5] Đang đẩy code lên nhánh main...
git push origin main
if %ERRORLEVEL% neq 0 (
    echo [Cảnh báo] Không thể push lên nhánh main. Vui lòng kiểm tra kết nối Git.
)

echo [4/5] Đang tạo và đẩy Git Tag %TAG_NAME% lên GitHub...
git tag -d %TAG_NAME% >nul 2>&1
git tag -a %TAG_NAME% -m "Release %TAG_NAME%"
git push origin %TAG_NAME%
if %ERRORLEVEL% neq 0 (
    echo [Lỗi] Không thể push tag %TAG_NAME% lên GitHub!
    pause
    goto MENU
)

echo.
echo =====================================================================
echo                      🎉 THÀNH CÔNG RỰC RỠ!
echo =====================================================================
echo Đã kích hoạt GitHub Actions để tự động build và phát hành %TAG_NAME%!
echo.
echo * Các bước theo dõi tiếp theo:
echo   1. Xem tiến trình build tự động:
echo      👉 https://github.com/nhattVim/QLCV/actions
echo.
echo   2. Khi build xong (khoảng 3-5 phút), bản cài đặt và tệp update sẽ xuất hiện tại:
echo      👉 https://github.com/nhattVim/QLCV/releases/tag/%TAG_NAME%
echo.
echo   3. Tất cả các máy tính đã cài app QLCV khi mở lên sẽ tự động hiện
echo      nút thông báo cập nhật lên bản %TAG_NAME% trên thanh tiêu đề!
echo =====================================================================
echo.
pause
goto MENU

:DELETE_RELEASE
cls
echo =====================================================================
echo                 [2] XÓA BẢN PHÁT HÀNH / GIT TAG
echo =====================================================================
echo.
echo Nhập số phiên bản / tag muốn xóa (Ví dụ: 1.0.1 hoặc v1.0.1):
set /p DEL_VER="> Version / Tag: "

if "%DEL_VER%"=="" (
    echo [Lỗi] Bạn chưa nhập tag cần xóa.
    pause
    goto MENU
)

set "CLEAN_DEL=%DEL_VER%"
if "%CLEAN_DEL:~0,1%"=="v" set "CLEAN_DEL=%CLEAN_DEL:~1%"
if "%CLEAN_DEL:~0,1%"=="V" set "CLEAN_DEL=%CLEAN_DEL:~1%"
set "DEL_TAG=v%CLEAN_DEL%"

echo.
echo CẢNH BÁO: Thao tác này sẽ xóa tag %DEL_TAG% ở cả máy này và trên GitHub!
set /p CONFIRM_DEL="Bạn có chắc chắn 100%% muốn xóa %DEL_TAG%? (y/n): "
if /i not "%CONFIRM_DEL%"=="y" goto MENU

echo.
echo [1/3] Đang xóa local tag %DEL_TAG%...
git tag -d %DEL_TAG%

echo [2/3] Đang xóa remote tag %DEL_TAG% trên GitHub...
git push origin --delete %DEL_TAG%

echo [3/3] Đang kiểm tra xóa GitHub Release...
where gh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    gh release delete %DEL_TAG% --yes --cleanup-tag >nul 2>&1
    echo [OK] Đã xóa bản Release trên GitHub thông qua GitHub CLI.
) else (
    echo.
    echo [Lưu ý] Máy chưa cài GitHub CLI (gh).
    echo Để xóa hẳn Release trên giao diện web GitHub, vui lòng truy cập:
    echo 👉 https://github.com/nhattVim/QLCV/releases
    echo Bấm vào bản %DEL_TAG% -> Bấm icon 🗑️ (Delete) để xóa.
)

echo.
echo =====================================================================
echo                    ĐÃ XÓA HOÀN TẤT TAG %DEL_TAG%!
echo =====================================================================
echo.
pause
goto MENU

:EXIT
exit /b 0
