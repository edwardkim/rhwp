# 실제 데스크톱 texture 검증

이 도구는 bounded 참조 plugin을 검증하는 별도 runner입니다. 제품의 렌더링 코드, 공개 Flutter
완료 계약, 네 개 불변·quarantine 프레임 한도와 기본 비활성 Metal 피처를 변경하지 않습니다.

#8739의 Windows SDK 빌드·D3D12 producer 검증을 보존합니다. 다음 단계는 실제 Flutter
consumer가 의도한 revision을 화면에 표시했는지 확인하는 것입니다. DXGI descriptor를 등록하거나
열었다는 사실만으로 compositor가 해당 이미지를 읽었다고 판단하지 않습니다.

## Windows 빌드와 화면 수집

실제 Windows의 대화형 desktop, 준비된 Flutter SDK, Python 3이 설치된 Visual Studio developer
shell과 서로 일치하는 Native D3D DLL·빌드 영수증이 필요합니다.
`mydocs/pr/assets/seo-rii-20261107/platform-execution/windows-final-correction.json`은 영수증
스키마의 예시이며 다른 DLL의 검증을 대신하지 않습니다. 먼저 검증할 소스를 커밋합니다.
도구는 추적되는 참조 패키지와 C header를 새 디렉터리에 복사하여 별도 runner를 만듭니다.
저장소의 기존 plugin·example 위에 `flutter create`를 실행하지 않습니다. 복사한 Dart/C와 Windows
C++·header·CMake 및 Native C header 등 추적 입력 전체를 빌드 후 다시 검사합니다. 별도 생성된
runner 파일은 이 원본 패키지 입력과 구분합니다.

```powershell
python bindings/Flutter/tool/build_desktop_probe.py `
  --flutter-sdk C:/SDK/flutter `
  --native C:/artifacts/rhwp_native_ffi.dll `
  --native-receipt C:/artifacts/windows-native-build.json `
  --output C:/Temp/rhwp-consumer-build
```

빌드 도구는 참조 소스·Native 소스 SHA, Flutter·engine revision, 소스별 hash와 runner·plugin·
dispatcher·DLL·입력 hash를 기록합니다. debug runner가 실제로 실행하는
`data/flutter_assets/kernel_blob.bin`을 필수로 포함해 실행 파일 디렉터리 전체의 상대 경로·hash·
바이트 manifest를 기록합니다. runtime bundle은 최대 1,024개 파일·4,096개 entry·1 GiB이며
symlink/reparse point·경로 이탈·빠진/추가/변경된 파일을 거절합니다. Windows runner·그에 딸린
의존 파일과 data를 함께 옮긴 경우에만 같은 manifest로 재검증할 수 있습니다. 빌드 영수증은
512 KiB 이하로 제한하고 capture 출력은 runtime bundle 밖에 둡니다.
실행 파일 옆의 실제 plugin DLL과 `flutter_windows.dll`도
별도 artifact로 기록하며 수집 시작·완료 때 hash를 다시 확인합니다. 실행 파일을 다른 경로로
옮긴 경우 원래 영수증 경로 대신 새 실행 파일 옆에서 실제로 로드할 DLL을 검사합니다. 각 명령은 시작할 때 로그와 PID를 남기며 종료 상태를
`build-receipt.json`에 기록합니다. 완료할 때까지 원본 소스를 유지하고, 명령이 종료된 뒤에
완료 로그를 확인합니다. 기존 출력 디렉터리는 덮어쓰지 않고 거절합니다.

영수증의 실제 artifact 경로를 사용하여 실행합니다.

```powershell
python bindings/Flutter/tool/qualify_desktop_texture.py `
  --runner C:/Temp/rhwp-consumer-build/runner/build/windows/x64/runner/Debug/rhwp_texture_qualification.exe `
  --native C:/artifacts/rhwp_native_ffi.dll `
  --dispatcher C:/Temp/rhwp-consumer-build/rhwp_flutter_owner.dll `
  --fixture C:/Temp/rhwp-consumer-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx `
  --build-receipt C:/Temp/rhwp-consumer-build/build-receipt.json `
  --cpu-reference `
  --output C:/Temp/rhwp-consumer-capture
```

`--timeout`은 stage 대기뿐 아니라 CPU 비교·두 실제 held capture·acknowledgement·종료 대기와
마지막 runtime identity 검증에도 적용합니다. 이미 생성된 stage라도 deadline 뒤에는 캡처하거나
성공으로 확인하지 않습니다. 실패한 실제 raw는 보존하고 소유 runner를 정리합니다.

probe 창은 다른 창에 가려지지 않은 전경으로 유지합니다. 수집기는 per-monitor-v2 물리 좌표와
Win32의 실제 desktop 화면을 사용합니다. `PrintWindow`, 가짜 registrar, widget screenshot이나
producer readback으로 대체하지 않습니다. 원 문서→revision 변경→viewport resize의 세 프레임을
유지한 상태로 검사합니다. 각 단계는 nonce·실행 PID와 document/revision/font/view/device/DPR
stamp를 검증하며, 같은 불변 snapshot을 Native CPU batch API로 독립 재생합니다. GPU의 정상
표시 경로에 whole-frame readback을 추가하지 않습니다. CPU 기준 재생과 compositor 화면 읽기는
검증에만 발생하는 비용입니다.

viewport에는 불투명한 clear·종이 배경을 사용합니다. CPU의 균일한 내부 pixel은 8비트 채널 값
1 이내여야 합니다. 나머지 차이는 CPU 경계에서 한 pixel 이내와 그 주변 색상 범위에 있어야 합니다.
빈 cover·오래된 이미지·누락된 내용·균일 영역의 잘못된 색·source 변경·불안정한 화면은 실패합니다.
`--cpu-reference`는 같은 immutable snapshot·full stamp를 유지한 독립 2–4배 CPU 재생을
명시적으로 선택합니다. target/placement만 확대하고 4,096 dimension·4 MP·16 MiB를 넘으면
실패합니다. 저해상도 CPU 3×3 box가 표현하지 못하는 얇은 선의 coverage는 독립 source palette의
13 RGB projection으로 검사하며, 균일 내부 ±1·한 pixel 경계 지원과 유일한 zero-shift 정렬을
유지합니다. GPU 화면에서 expected 색상을 만들지 않습니다. 옵션을 생략하면 기존 strict
저해상도 비교 계약을 유지합니다. 실패한 실제 compositor raw도 비교 전에 보존합니다.
이는 명시된 비교 계약이며 CPU/GPU 안티앨리어싱의 byte 단위 동일성이나 최종 한컴 일치 판정이 아닙니다.

세 화면을 수집한 뒤 실제 controller를 mount한 상태로 닫습니다. Native의 source/snapshot/frame/
lease 계수는 0이어야 하며 producer 종료와 실제 등록 해제를 확인해야 합니다. 게시한 GPU target은
bounded 불변 저장소에 남습니다. close·unregister를 근거로 프레임 credit을 환급하면 실패합니다.

영수증의 성공 범위는 `bounded-windows-consumer-passed`입니다. `consumerGpuCompletionVerified:false`,
`continuousPresentationSupported:false`, `hardwareAccelerationVerified:false`를 유지합니다.
세 불변 프레임의 표시 성공은 consumer queue fence·안전한 재사용·하드웨어 성능·장치 상실·장시간
renderer나 실물 OS IME/접근성 검증을 뜻하지 않습니다. 연속 backend에는 별도로 검증한 consumer
수명 계약이 필요하며 embedder·engine 연결이 필요할 수도 있습니다.

## 공개 입력과 Linux에서 가능한 음성 대조

`test/fixtures/desktop-consumer.hwpx`는 기존 공개 입력
`tests/fixtures/pr8061_cell_picture_layer/bordered-page-picture.hwpx`에서 section 설정을 보존하고
그림을 포함한 표 앞에 빈 본문 문단 하나를 둔 8,082바이트 HWPX입니다. 원본 picture bytes와
그림·표 속성은 보존합니다. SHA-256은
`648dade82585fa2845b80ec510b5f55c67e8b6127688dd3748b9a2068d87a3d9`입니다.

실제 probe는 첫 빈 문단에 여섯 개의 개행을 넣어 그림을 이동시킵니다. 시스템 글꼴의 존재와
관계없이 revision 변경이 화면에서 관측되어야 합니다. CPU 기준에 충분한 어두운 내용과
다양한 색상이 있어야 하며 두 revision의 pixel hash도 달라야 합니다. 빈 출력이나 내용 변화가
없는 실행은 표시 성공으로 처리하지 않습니다.

```bash
python3 -B test/desktop_qualification_contract.py
```

이식 가능한 검사는 nonce·PID·전체 stamp·DPR 거절, byte 한도, 잘못된 pixel·빈 cover·stale 화면,
등록 해제 누락과 credit 환급 금지를 검사합니다. 실제 Windows SDK나 Flutter D3D consumer를
실행한 결과가 아닙니다. 실제 빌드·화면 수집 도구는 Linux를 거절하며, Windows Python을 쓰는 Wine도
`ntdll.wine_get_version` export로 확인해 거절합니다. Dart kernel·data나 plugin/engine DLL을
변경한 결과를 이전 source/engine 영수증으로 처리하지 않습니다. NT loader를 확인할 수 없는 경우도
실행을 보류합니다. 이 guard의 대역 검사는 실제 Windows 실행 증거가 아닙니다.

`test/desktop_oracle_test.dart`는 실제 Native/owner 라이브러리를 지정했을 때만 CPU oracle의
동일 snapshot 재생·독립 Dart byte 복사·frame/lease 해제와 편집 후 stale 거절을 실행합니다.
이 검사는 Native CPU 경계의 검증이며 Windows 화면 수집과 구분합니다.

이 구현은 Linux에서 코드 검토와 portable 검사를 수행할 수 있습니다. 실제 Windows runner의
빌드·화면 수집은 해당 target desktop에서 기록한 artifact로 실행하기 전까지 미검증입니다.
Apple SDK/Metal은 별도의 target 검증 단계입니다.
