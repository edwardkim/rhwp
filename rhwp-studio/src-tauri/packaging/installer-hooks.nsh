; RHWP Studio NSIS 설치 훅
;
; 자체 서명 인증서를 "신뢰할 수 있는 게시자"와 "신뢰할 수 있는 루트 인증 기관" 저장소에 등록해,
; 설치 이후 실행 시 SmartScreen이 게시자 미확인으로 표시하지 않도록 한다.
; 사내에 AD/GPO 일괄 배포 체계가 없어(설계 스펙 참고) 이 등록을 설치 과정 안에 포함시켰다.
;
; 인증서 등록이 실패해도(UAC 거부, certutil 부재 등) 설치 자체는 계속 진행한다 — ExecWait는
; 반환 코드를 $0에 저장할 뿐 실패 시 스크립트를 중단시키지 않는다.

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "RHWP Studio 인증서를 신뢰할 수 있는 게시자로 등록합니다..."
  ExecWait '"$SYSDIR\certutil.exe" -addstore "TrustedPublisher" "$INSTDIR\rhwp-studio-cert.cer"' $0
  ExecWait '"$SYSDIR\certutil.exe" -addstore "Root" "$INSTDIR\rhwp-studio-cert.cer"' $0
!macroend
