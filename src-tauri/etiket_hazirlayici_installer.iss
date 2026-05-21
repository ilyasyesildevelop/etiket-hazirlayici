[Setup]
AppName=Etiket Hazırlayıcı
AppVersion=26.6.0
DefaultDirName={autopf}\Etiket Hazırlayıcı
DefaultGroupName=Etiket Hazırlayıcı
DisableProgramGroupPage=no
DisableReadyPage=yes
AllowNoIcons=yes
CreateAppDir=yes
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=..\dist
OutputBaseFilename=EtiketHazirlayiciSetup
SetupIconFile=icons\icon.ico
UninstallDisplayIcon={app}\etiket_hazirlayici.exe
Uninstallable=yes

#define SourceExePath "target\\release\\etiket-hazirlayici.exe"

[Files]
Source: "{#SourceExePath}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Etiket Hazırlayıcı"; Filename: "{app}\etiket_hazirlayici.exe"
Name: "{group}\Kaldır Etiket Hazırlayıcı"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\etiket_hazirlayici.exe"; Description: "Etiket Hazırlayıcı'yı başlat"; Flags: nowait postinstall skipifsilent
