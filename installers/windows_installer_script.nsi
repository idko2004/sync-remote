OutFile "sync-remote-windows-installer.exe"
RequestExecutionLevel admin

Function .onInit
	IfFileExists "$PROGRAMFILES64\*" 0 +2
		StrCpy $INSTDIR "$PROGRAMFILES64\idko2004.github.io\sync-remote"
	IfFileExists "$INSTDIR\*" 0 +2
		Return
	StrCpy $INSTDIR "$PROGRAMFILES\idko2004.gitub.io\sync-remote"
FunctionEnd

Page directory
Page instfiles

Section
	SetOutPath $INSTDIR
	File target\release\sync-remote.exe
	
	CreateShortcut "$SMPROGRAMS\sync-remote.lnk" "wt" 'cmd /k "$INSTDIR\sync-remote.exe" && pause && exit'
SectionEnd