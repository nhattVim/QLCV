; Script Inno Setup de tao bo cai dat QLCV_Setup.exe chuyen nghiep
#define MyAppName "QLCV"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "QLCV Team"
#define MyAppExeName "QLCV.exe"

[Setup]
AppId={{8F5B8B12-4C13-4E8E-927A-D48366112C7D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={userappdata}\Programs\QLCV
DefaultGroupName={#MyAppName}
OutputDir=dist
OutputBaseFilename=QLCV_Setup
Compression=lzma2/max
SolidCompression=yes
PrivilegesRequired=lowest
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "dist\QLCV.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Go bo cai dat"; Filename: "{uninstallexe}"
Name: "{userdesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
