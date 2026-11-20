; RustView installer (Inno Setup 6). Build: iscc /DAppVersion=0.1.0 installer\rustview.iss
#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#define AppName "RustView"
#define AppExe "rustview.exe"
#define ProgId "RustView.Image"

[Setup]
AppId={{B7C3A0E2-5D41-4F8E-9A6B-2E1D7C94F3A1}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=RustView
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=RustView-Setup-{#AppVersion}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\{#AppExe}
LicenseFile=..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Per-user install by default (no admin rights); the user can pick "for all users" in a dialog.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked
Name: "assoc"; Description: "Open image files (GIF, WebP, PNG, JPG, ...) with {#AppName}"; GroupDescription: "File associations:"

[Files]
Source: "..\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; ProgID: icon + "open" command. The icon here is what Explorer shows for associated files.
Root: HKA; Subkey: "Software\Classes\{#ProgId}"; ValueType: string; ValueData: "Image ({#AppName})"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\{#ProgId}\DefaultIcon"; ValueType: string; ValueData: "{app}\icon.ico"
Root: HKA; Subkey: "Software\Classes\{#ProgId}\shell\open\command"; ValueType: string; ValueData: """{app}\{#AppExe}"" ""%1"""

; Make the app appear in "Open with" and in Settings > Default apps.
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\shell\open\command"; ValueType: string; ValueData: """{app}\{#AppExe}"" ""%1"""; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\{#AppName}\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#AppName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\{#AppName}\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Lightweight image viewer"
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#AppName}"; ValueData: "Software\{#AppName}\Capabilities"; Flags: uninsdeletevalue

; Supported extensions (keep in sync with EXTENSIONS in src/main.rs)
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".jpg"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.jpg\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.jpg"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".jpeg"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.jpeg\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.jpeg"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".jpe"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.jpe\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.jpe"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".jfif"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.jfif\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.jfif"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".png"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.png\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.png"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".gif"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.gif\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.gif"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".webp"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.webp\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.webp"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".bmp"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.bmp\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.bmp"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".ico"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.ico\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.ico"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".tif"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.tif\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.tif"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".tiff"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.tiff\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.tiff"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".tga"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.tga\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.tga"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".qoi"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.qoi\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.qoi"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pnm"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.pnm\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.pnm"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".ppm"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.ppm\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.ppm"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pgm"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.pgm\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.pgm"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pbm"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.pbm\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.pbm"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\{#AppName}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pam"; ValueData: "{#ProgId}"
Root: HKA; Subkey: "Software\Classes\.pam\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.pam"; ValueType: string; ValueData: "{#ProgId}"; Tasks: assoc; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
; Windows 10/11 protects the user's default-app choice, so offer to open the Default apps page.
Filename: "ms-settings:defaultapps"; Description: "Open Windows Default apps settings (choose {#AppName} for images)"; Flags: shellexec postinstall skipifsilent unchecked
