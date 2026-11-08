# Windows 불변 target 연속 표시 검증

`tool/qualify_windows_continuous.py`는 실제 Windows의 대화형 Flutter desktop에서 기본 32회
게시를 검사한다. [기존 빌드 도구](GPU_QUALIFICATION.md#windows-빌드와-화면-수집)로 커밋된
공통 probe·Windows plugin을 별도 디렉터리에 빌드한 뒤 그 영수증과 artifact를 전달한다.
Linux와 Wine은 실제 수집 대상으로 거절한다. 아래 portable 검사는 실제 Windows 실행 증거가 아니다.

```powershell
python bindings/Flutter/tool/qualify_windows_continuous.py `
  --runner C:/Temp/rhwp-consumer-build/runner/build/windows/x64/runner/Debug/rhwp_texture_qualification.exe `
  --native C:/artifacts/rhwp_native_ffi.dll `
  --dispatcher C:/Temp/rhwp-consumer-build/rhwp_flutter_owner.dll `
  --fixture C:/Temp/rhwp-consumer-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx `
  --build-receipt C:/Temp/rhwp-consumer-build/build-receipt.json `
  --output C:/Temp/rhwp-continuous-capture --frames 32 --timeout 600
```

제한 시간은 stage 대기·CPU 비교·두 held capture·acknowledgement·종료와 최종 runtime identity
검증을 모두 포함한다. 준비된 stage라도 deadline 뒤에는 승인하거나 성공으로 기록하지 않는다.
실패한 실제 raw와 진단은 보존하고 소유 runner를 정리한다.

새 출력 디렉터리는 runtime bundle 밖에 둔다. frame 수는 32..64, 제한 시간은 30..600초다.
도구는 `backend:windowsImmutableTransfer`, `presentationKind:externalTexture`,
`frameCount:32`, `cpuReference:true`를 config로 전달한다. 기본 legacy backend의 세 프레임
검증 계약은 유지하며 이 별도 실행은 `immutableOwnershipTransfer`를 명시적으로 선택한다.
runner·plugin·Flutter engine·Native·dispatcher·fixture의 hash와 debug Dart kernel을 포함한
전체 runtime bundle을 수집 전후에 다시 검사한다. 영수증에 source/Native/Flutter/engine SHA와
collector hash, PID, nonce, 결과 및 종료 상태를 남긴다.

첫 화면, 실제 편집 revision, 물리 target resize 후 viewport를 바꾸며 연속 게시한다. 모든 단계에서
동일 등록의 accepted/rendered/descriptor-open serial과 전체 source stamp, 증가하는 view,
CPU 재생 snapshot이 Native accepted snapshot에 포함되는지 확인한다. 서로 이웃한 CPU hash도
달라야 한다. foreground Win32 desktop의 실제 pixel을 두 번 읽어 같은 화면이 유지되는지 검사한다.
widget screenshot, mock registrar, producer target readback을 화면 검증으로 사용하지 않는다.

같은 불변 snapshot·stamp를 CPU로 독립 재생하고 target/placement만 2..4배 확대하여 source
색상 지원을 얻는다. 확대 reference는 4 MP·16 MiB·한 변 4,096 pixel 이내이며 불가능하면 실패한다.
균일 내부 채널 차이는 1 이내다. 경계 차이는 CPU 경계의 한 pixel 지원과 독립 reference의 13방향
색상 projection 범위에 있어야 한다. 3×3 정수 이동 중 원래 위치가 유일한 최선이어야 한다.
GPU 캡처에서 expected 색상을 만들지 않는다. 빈 cover·stale 화면·1/2 pixel 이동·magenta/cyan
오염·경계 밖 halo·균일 내부 오차를 음성 대조로 거절한다.

Native 통계는 process 전체에서 생산자가 보유한 target 최대 4개·64 MiB, target 한 개 16 MiB,
descriptor ticket 최대 8개를 검사한다. import pending은 이 생산자 상한 안에 있어야 한다.
legacy retention·quarantine·whole-frame producer readback은 0이다. 현재 frame과 마지막으로
export한 frame의 참조가 유지되는 것은 허용한다. descriptor opening, CPU descriptor 읽기의
종료, 마지막 생산자 참조 해제가 서로 다른 시점임을 반영하여 누적 계수의 진행을 검사한다.
각 단계에서 이미 관측한 frame은 handoff 또는 생산자 참조로 계상되어야 한다.

닫은 뒤 `closed`, `ownerFinished`, 실제 `unregistered`가 모두 참이어야 한다. 생산자/import/
legacy/ticket 계수와 Native source/snapshot/frame/lease 계수는 모두 0이어야 한다.
누적 `handedOffFrames`는 관측한 게시 수에서 1을 뺀 값 이상이어야 한다. 마지막 frame의 pin은
등록 해제 때 정리될 수 있으므로 각 descriptor opening을 즉시 handoff로 세지 않는다.

성공 범위는 `windows-immutable-transfer-consumer-passed`다. 새 불변 target을 연속 표시했다는
검증이며 `consumerGpuCompletionVerified:false`, `targetReuse:false`,
`consumerAndDriverMemoryIncluded:false`, `hardwareAccelerationVerified:false`를 유지한다.
생산자 참조 상한은 소비자·driver의 전체 GPU 메모리 상한이나 consumer queue fence를 입증하지 않는다.

```bash
python3 -B bindings/Flutter/test/windows_continuous_qualification_contract.py
python3 -B bindings/Flutter/test/desktop_qualification_contract.py
```

portable 검사는 protocol·소유권·한도·pixel 거절 계약을 확인한다. 실제 32회 화면·닫기 검증의 완료는
진짜 Windows에서 생성한 `receipt.json`의 성공 결과와 artifact를 확인한 뒤에만 기록한다.
