!ifndef SYNTH_VERSION
  !define SYNTH_VERSION "0.1.0"
!endif
!ifndef SYNTH_VI_VERSION
  !define SYNTH_VI_VERSION "0.1.0.0"
!endif
!ifndef SYNTH_BIN
  !define SYNTH_BIN "dist\synth-windows-x86_64.exe"
!endif
!ifndef SYNTH_OUT
  !define SYNTH_OUT "dist\WebizenAI-Synth-Setup-windows-x86_64.exe"
!endif
!ifndef SYNTH_ICON
  !define SYNTH_ICON "assets\branding\icon.ico"
!endif
!ifndef SYNTH_SIDEBAR
  !define SYNTH_SIDEBAR "assets\branding\installer-sidebar.bmp"
!endif
!ifndef SYNTH_HEADER
  !define SYNTH_HEADER "assets\branding\installer-header.bmp"
!endif

Name "Webizen AI Synth"
OutFile "${SYNTH_OUT}"
InstallDir "$LOCALAPPDATA\Programs\Webizen AI Synth"
RequestExecutionLevel user
Unicode True
SetCompressor /SOLID lzma
Icon "${SYNTH_ICON}"
UninstallIcon "${SYNTH_ICON}"
BrandingText "Webizen AI Synth"
VIProductVersion "${SYNTH_VI_VERSION}"
VIAddVersionKey "ProductName" "Webizen AI Synth"
VIAddVersionKey "FileDescription" "Webizen AI Synth operator"
VIAddVersionKey "FileVersion" "${SYNTH_VERSION}"
VIAddVersionKey "ProductVersion" "${SYNTH_VERSION}"

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "WinMessages.nsh"

!define MUI_ICON "${SYNTH_ICON}"
!define MUI_UNICON "${SYNTH_ICON}"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${SYNTH_HEADER}"
!define MUI_WELCOMEFINISHPAGE_BITMAP "${SYNTH_SIDEBAR}"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\synth.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "operator"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath "$INSTDIR"
  File "/oname=synth.exe" "${SYNTH_BIN}"
  File "/oname=icon.ico" "${SYNTH_ICON}"
  CreateDirectory "$SMPROGRAMS\Webizen AI Synth"
  CreateShortCut "$SMPROGRAMS\Webizen AI Synth\Webizen AI Synth.lnk" "$INSTDIR\synth.exe" "operator" "$INSTDIR\icon.ico"
  CreateShortCut "$SMPROGRAMS\Webizen AI Synth\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  CreateShortCut "$DESKTOP\Webizen AI Synth.lnk" "$INSTDIR\synth.exe" "operator" "$INSTDIR\icon.ico"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "DisplayName" "Webizen AI Synth"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "DisplayIcon" "$INSTDIR\synth.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "Publisher" "Webizen AI"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "DisplayVersion" "${SYNTH_VERSION}"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth" "EstimatedSize" "$0"
  ReadRegStr $R0 HKCU "Environment" "Path"
  Push $R0
  Push "$INSTDIR"
  Call StrContains
  Pop $R1
  StrCmp $R1 "" 0 skip_path
    StrCmp $R0 "" 0 +3
      WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
      Goto skip_path
    WriteRegExpandStr HKCU "Environment" "Path" "$R0;$INSTDIR"
  skip_path:
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\Webizen AI Synth.lnk"
  Delete "$SMPROGRAMS\Webizen AI Synth\Webizen AI Synth.lnk"
  Delete "$SMPROGRAMS\Webizen AI Synth\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Webizen AI Synth"
  Delete "$INSTDIR\synth.exe"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\WebizenAISynth"
SectionEnd

Function StrContains
  Exch $R1
  Exch
  Exch $R2
  Push $R3
  Push $R4
  Push $R5
  StrLen $R3 $R1
  StrCpy $R4 0
loop:
  StrCpy $R5 $R2 $R3 $R4
  StrCmp $R5 $R1 found
  StrCmp $R5 "" notfound
  IntOp $R4 $R4 + 1
  Goto loop
found:
  StrCpy $R1 $R1
  Goto done
notfound:
  StrCpy $R1 ""
done:
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Exch $R1
FunctionEnd
