
@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul
title QLCV - Release Manager

cd /d "%~dp0.." || (
    echo [ERROR] Cannot access project directory.
    pause
    exit /b 1
)

set "REPO=nhattVim/QLCV"
set "MAIN_BRANCH=main"
set "RELEASE_URL=https://github.com/nhattVim/QLCV/releases"
set "ACTIONS_URL=https://github.com/nhattVim/QLCV/actions"

:MENU
cls
call :HEADER "QLCV RELEASE MANAGER"
echo.
echo   [1]  PHAT HANH PHIEN BAN MOI
echo        Cap nhat version, commit, push code va tao Git Tag
echo.
echo   [2]  XOA RELEASE / GIT TAG
echo        Xoa GitHub Release va tag local/remote
echo.
echo   [3]  KIEM TRA TRANG THAI
echo        Kiem tra Git, branch, remote va GitHub CLI
echo.
echo   [4]  MO GITHUB ACTIONS
echo.
echo   [5]  THOAT
echo.
call :FOOTER
set "CHOICE="
set /p "CHOICE=  Lua chon [1-5]: "

if "!CHOICE!"=="1" goto RELEASE
if "!CHOICE!"=="2" goto DELETE_RELEASE
if "!CHOICE!"=="3" goto STATUS
if "!CHOICE!"=="4" start "" "!ACTIONS_URL!" & goto MENU
if "!CHOICE!"=="5" goto EXIT
goto MENU


:RELEASE
cls
call :HEADER "PHAT HANH PHIEN BAN MOI"
echo.
echo  Quy trinh:
echo  1. Kiem tra moi truong va Git Tag
echo  2. Cap nhat version
echo  3. Commit va push main
echo  4. Tao Git Tag va kich hoat GitHub Actions
echo.

call :CHECK_GIT
if errorlevel 1 goto PAUSE_MENU

call :CHECK_NODE
if errorlevel 1 goto PAUSE_MENU

for /f "delims=" %%B in ('git branch --show-current') do set "CURRENT_BRANCH=%%B"
if /i not "!CURRENT_BRANCH!"=="%MAIN_BRANCH%" (
    echo [ERROR] Ban dang o branch "!CURRENT_BRANCH!".
    echo Hay chuyen sang branch %MAIN_BRANCH% truoc khi phat hanh.
    goto PAUSE_MENU
)

echo Dang dong bo Git Tag tu GitHub...
git fetch --tags origin
if errorlevel 1 (
    echo [ERROR] Khong the dong bo Git Tag.
    goto PAUSE_MENU
)

set "TAG_COUNT=0"
for /f "delims=" %%T in ('git tag -l') do (
    set /a TAG_COUNT+=1
    set "TAG_!TAG_COUNT!=%%T"
)

echo.
echo  Git Tag hien co:
echo  -------------------------------------------------------------------
if !TAG_COUNT! equ 0 (
    echo   Chua co tag nao.
) else (
    call :PRINT_TAGS !TAG_COUNT!
)
echo  -------------------------------------------------------------------
echo.

:PROMPT_VERSION
set "INPUT_VER="
set "CLEAN_VER="
set "TAG_NAME="

set /p "INPUT_VER=  Nhap version moi (vd: 1.0.1 hoac v1.0.1, 0 de quay lai): "

if not defined INPUT_VER (
    echo [ERROR] Version khong duoc de trong.
    goto PROMPT_VERSION
)
if "!INPUT_VER!"=="0" goto MENU

set "CLEAN_VER=!INPUT_VER!"
if /i "!CLEAN_VER:~0,1!"=="v" set "CLEAN_VER=!CLEAN_VER:~1!"

rem Chi chap nhan version X.Y.Z, vi du 1.0.1
node -e "if(!/^[0-9]+\.[0-9]+\.[0-9]+$/.test(process.argv[1]||''))process.exit(1)" "!CLEAN_VER!" >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Version khong hop le. Hay nhap X.Y.Z, vi du 1.0.1.
    goto PROMPT_VERSION
)

set "TAG_NAME=v!CLEAN_VER!"

rem Kiem tra tag local
git tag -l "!TAG_NAME!" | findstr /x /l /c:"!TAG_NAME!" >nul
if not errorlevel 1 (
    echo [ERROR] Tag !TAG_NAME! da ton tai tren may.
    goto PAUSE_MENU
)

rem Kiem tra tag remote
set "REMOTE_TAG="
for /f "tokens=1" %%H in ('git ls-remote --tags origin "refs/tags/!TAG_NAME!" 2^>nul') do (
    set "REMOTE_TAG=%%H"
)
if defined REMOTE_TAG (
    echo [ERROR] Tag !TAG_NAME! da ton tai tren GitHub.
    goto PAUSE_MENU
)

echo.
echo  Version moi : !CLEAN_VER!
echo  Git Tag     : !TAG_NAME!
echo  Branch      : !MAIN_BRANCH!
echo.
echo  LUU Y: Script se commit cac thay doi trong repository.
echo  Hay dam bao khong co file rieng tu hoac secret chua duoc bo qua.
echo.
set "CONFIRM="
set /p "CONFIRM=  Xac nhan phat hanh? (Y/N): "
if /i not "!CONFIRM!"=="Y" goto MENU

echo.
echo [1/5] Cap nhat version...

node -e "const fs=require('fs'); const files=['package.json','src-tauri/tauri.conf.json']; const data=files.map(p=>{if(!fs.existsSync(p))throw Error('Missing file: '+p);return [p,JSON.parse(fs.readFileSync(p,'utf8'))]}); const lock='package-lock.json'; let l=null; if(fs.existsSync(lock))l=JSON.parse(fs.readFileSync(lock,'utf8')); const v=process.argv[1]; for(const [p,j] of data)j.version=v; if(l){l.version=v;if(l.packages&&l.packages[''])l.packages[''].version=v;} for(const [p,j] of data)fs.writeFileSync(p,JSON.stringify(j,null,2)+'\n','utf8'); if(l)fs.writeFileSync(lock,JSON.stringify(l,null,2)+'\n','utf8');" "!CLEAN_VER!"

if errorlevel 1 (
    echo [ERROR] Cap nhat version that bai.
    goto PAUSE_MENU
)

echo [2/5] Kiem tra va commit thay doi...
git add -A
if errorlevel 1 (
    echo [ERROR] git add that bai.
    goto PAUSE_MENU
)

git commit -m "release: !TAG_NAME!"
if errorlevel 1 (
    echo [ERROR] Commit that bai. Khong tiep tuc phat hanh.
    echo Kiem tra Git identity, hook va trang thai repository.
    goto PAUSE_MENU
)

echo [3/5] Push code len origin/%MAIN_BRANCH%...
git push origin %MAIN_BRANCH%
if errorlevel 1 (
    echo [ERROR] Push main that bai. Khong tao tag.
    goto PAUSE_MENU
)

echo [4/5] Tao Git Tag !TAG_NAME!...
git tag -a "!TAG_NAME!" -m "Release !TAG_NAME!"
if errorlevel 1 (
    echo [ERROR] Khong tao duoc Git Tag.
    goto PAUSE_MENU
)

echo [5/5] Push Git Tag len GitHub...
git push origin "!TAG_NAME!"
if errorlevel 1 (
    echo [ERROR] Push tag that bai.
    echo Tag local duoc giu lai de ban kiem tra.
    goto PAUSE_MENU
)

echo.
call :HEADER "PHAT HANH THANH CONG"
echo.
echo  Version : !CLEAN_VER!
echo  Tag     : !TAG_NAME!
echo.
echo  Theo doi build:
echo  !ACTIONS_URL!
echo.
echo  Release:
echo  !RELEASE_URL!/tag/!TAG_NAME!
echo.
echo  LUU Y:
echo  - Kiem tra GitHub Actions build thanh cong.
echo  - Kiem tra file cai dat trong GitHub Release.
echo  - Kiem tra updater tren may da cai ban cu.
echo.
call :FOOTER
pause
goto MENU


:DELETE_RELEASE
cls
call :HEADER "XOA RELEASE / GIT TAG"
echo.
echo  CANH BAO: Thao tac nay co the xoa release va tag tren GitHub.
echo.

call :CHECK_GIT
if errorlevel 1 goto PAUSE_MENU

where gh >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Chua cai GitHub CLI (gh).
    echo Can cai dat: https://cli.github.com/
    echo Sau do dang nhap bang lenh: gh auth login
    goto PAUSE_MENU
)

gh auth status >nul 2>&1
if errorlevel 1 (
    echo [ERROR] GitHub CLI chua dang nhap hoac khong co quyen.
    echo Hay chay: gh auth login
    goto PAUSE_MENU
)

echo Dang dong bo Git Tag...
git fetch --tags origin
if errorlevel 1 (
    echo [ERROR] Khong the dong bo Git Tag.
    goto PAUSE_MENU
)

set "TAG_COUNT=0"
for /f "delims=" %%T in ('git tag -l') do (
    set /a TAG_COUNT+=1
    set "TAG_!TAG_COUNT!=%%T"
)

if !TAG_COUNT! equ 0 (
    echo Khong tim thay Git Tag.
    goto PAUSE_MENU
)

echo.
call :PRINT_TAGS !TAG_COUNT!
echo.
echo  [0]   Quay lai
echo  [ALL] Chon tat ca tag
echo.
set "SEL="
set /p "SEL=  Nhap so thu tu (vd: 1 3 hoac 1,3 hoac ALL): "

if not defined SEL goto MENU
if "!SEL!"=="0" goto MENU

set "SELECTED_TAGS="
if /i "!SEL!"=="ALL" (
    call :SELECT_ALL_TAGS !TAG_COUNT!
) else (
    set "NORMALIZED_SEL=!SEL:,= !"
    for %%N in (!NORMALIZED_SEL!) do call :ADD_SELECTED %%N
)

if not defined SELECTED_TAGS (
    echo [ERROR] Khong co lua chon hop le.
    goto PAUSE_MENU
)

echo.
echo  Cac tag duoc chon:
for %%T in (!SELECTED_TAGS!) do echo    - %%T
echo.
echo  Moi GitHub Release tuong ung se duoc xoa neu ton tai.
echo  Git Tag local va remote cung se duoc xoa.
echo.
set "CONFIRM_DEL="
set /p "CONFIRM_DEL=  Nhap DELETE de xac nhan: "
if /i not "!CONFIRM_DEL!"=="DELETE" goto MENU

set "DELETE_ERRORS=0"
for %%T in (!SELECTED_TAGS!) do call :DELETE_ONE "%%T"

echo.
if "!DELETE_ERRORS!"=="0" (
    call :HEADER "HOAN TAT"
    echo Tat ca thao tac xoa da thanh cong.
) else (
    call :HEADER "HOAN TAT CO LOI"
    echo Mot hoac nhieu thao tac xoa that bai.
    echo Hay kiem tra thong bao va trang thai GitHub.
)
echo.
pause
goto MENU


:DELETE_ONE
set "DEL_TAG=%~1"
echo.
echo  -- Xu ly tag !DEL_TAG!

rem Xoa GitHub Release neu co
gh release view "!DEL_TAG!" >nul 2>&1
if not errorlevel 1 (
    gh release delete "!DEL_TAG!" --yes
    if errorlevel 1 (
        echo [ERROR] Khong xoa duoc GitHub Release !DEL_TAG!.
        set "DELETE_ERRORS=1"
    ) else (
        echo [OK] Da xoa GitHub Release !DEL_TAG!.
    )
) else (
    echo [INFO] Khong tim thay release hoac khong doc duoc release.
)

rem Xoa tag remote
git ls-remote --exit-code --tags origin "refs/tags/!DEL_TAG!" >nul 2>&1
if not errorlevel 1 (
    git push origin --delete "!DEL_TAG!"
    if errorlevel 1 (
        echo [ERROR] Khong xoa duoc tag remote !DEL_TAG!.
        set "DELETE_ERRORS=1"
    ) else (
        echo [OK] Da xoa tag remote !DEL_TAG!.
    )
) else (
    echo [INFO] Tag remote khong ton tai hoac khong truy cap duoc.
)

rem Xoa tag local
git tag -d "!DEL_TAG!" >nul 2>&1
if errorlevel 1 (
    echo [INFO] Khong xoa duoc tag local hoac tag khong ton tai.
) else (
    echo [OK] Da xoa tag local !DEL_TAG!.
)
exit /b


:STATUS
cls
call :HEADER "KIEM TRA TRANG THAI"
echo.
echo  [Git]
git --version
if errorlevel 1 echo [ERROR] Git chua duoc cai dat.

echo.
echo  [Node.js]
node --version
if errorlevel 1 echo [ERROR] Node.js chua duoc cai dat.

echo.
echo  [Repository]
git rev-parse --show-toplevel
if errorlevel 1 (
    echo [ERROR] Thu muc hien tai khong phai Git repository.
    goto PAUSE_MENU
)

echo.
echo  [Branch]
git branch --show-current

echo.
echo  [Remote]
git remote -v

echo.
echo  [GitHub CLI]
where gh >nul 2>&1
if errorlevel 1 (
    echo Chua cai dat gh.
) else (
    gh --version
    gh auth status
)

echo.
call :FOOTER
pause
goto MENU


:CHECK_GIT
git rev-parse --is-inside-work-tree >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Thu muc hien tai khong phai Git repository.
    exit /b 1
)

git remote get-url origin >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Khong tim thay Git remote origin.
    exit /b 1
)

git ls-remote origin HEAD >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Khong ket noi duoc GitHub qua origin.
    exit /b 1
)
exit /b 0


:CHECK_NODE
where node >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Chua cai Node.js.
    exit /b 1
)
node --version
exit /b 0


:PRINT_TAGS
for /l %%I in (1,1,%1) do echo   [%%I] !TAG_%%I!
exit /b 0


:SELECT_ALL_TAGS
for /l %%I in (1,1,%1) do (
    set "SELECTED_TAGS=!SELECTED_TAGS! !TAG_%%I!"
)
exit /b 0


:ADD_SELECTED
set "INDEX=%~1"
for /f "delims=0123456789" %%A in ("!INDEX!") do exit /b 0
if not defined TAG_%INDEX% exit /b 0
set "SELECTED_TAGS=!SELECTED_TAGS! !TAG_%INDEX%!"
exit /b 0


:HEADER
echo =====================================================================
echo   %~1
echo =====================================================================
exit /b 0


:FOOTER
echo =====================================================================
exit /b 0


:PAUSE_MENU
echo.
pause
goto MENU


:EXIT
endlocal
exit /b 0
