Name "sync-remote"
OutFile "sync-remote-windows-installer.exe"
RequestExecutionLevel admin

!include "LogicLib.nsh"
!include "FileFunc.nsh"

Function .onInit
	IfFileExists "$PROGRAMFILES64\*" 0 +2
		StrCpy $1 "$PROGRAMFILES64"
	IfFileExists "$1\*" 0 +2
		goto end
	StrCpy $1 "$PROGRAMFILES"

	end:
		StrCpy $INSTDIR "$1\idko2004.github.io\sync-remote"
FunctionEnd

Page directory
Page instfiles

UninstPage uninstConfirm
UninstPage instfiles

Section
    ; Obtener ruta completa de wt.exe
    StrCpy $0 "$LOCALAPPDATA\Microsoft\WindowsApps\wt.exe"

    ; Verificar si el archivo existe
    ${If} ${FileExists} $0
		DetailPrint "Windows Terminal is already installed."
    ${Else}
        MessageBox MB_YESNO "sync-remote works better with Windows Terminal. Open Microsoft Store to install it?" IDYES openStore

        Goto done

        openStore:
			DetailPrint "Please proceed to install Windows Terminal through the Microsoft Store."
            ExecShell "open" "ms-windows-store://pdp/?productid=9N0DX20HK701"
        done:
    ${EndIf}

	SetOutPath $INSTDIR
	File target\release\sync-remote.exe

	CreateShortcut "$SMPROGRAMS\sync-remote.lnk" "wt" 'cmd /k "$INSTDIR\sync-remote.exe" && pause && exit'
	
	WriteUninstaller "$INSTDIR\uninstall-sync-remote.exe"
	
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\sync-remote.idko2004.github.io" "DisplayName" "sync-remote"
	WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\sync-remote.idko2004.github.io" "UninstallString" "$INSTDIR\uninstall-sync-remote.exe"
SectionEnd

Section Uninstall
	MessageBox MB_YESNO "Remove user configuration and backups?" IDYES removeData

	Goto continue

	removeData:
		DetailPrint "Removing user data..."
		Delete "$LOCALAPPDATA\idko2004.github.io\sync-remote\*"
		RMDir "$LOCALAPPDATA\idko2004.github.io\sync-remote\"
	continue:

	DetailPrint "Uninstalling..."
	Delete "$INSTDIR\*"
	RMDir "$INSTDIR"
	Delete "$SMPROGRAMS\sync-remote.lnk"
	DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\sync-remote.idko2004.github.io"
SectionEnd

