# Native·Qt·Flutter 호스트 검증 중간 기록

## Windows 호스트 자원 사용과 보정

작업지시자는 Windows 11 호스트의 자원 사용을 지시했다. WSL의 Linux 검사를 Windows 검사로 대신하지 않고, 실제 Visual Studio 2022 17.14.36717.8 / MSVC / Windows SDK 10.0.26100.0과 Flutter 3.47.9의 Windows 공개 header·import library를 사용했다. 도구는 작업 전용 NTFS 임시 경로에만 준비했고 전역 PATH는 바꾸지 않았다.

- 원 `65a3a2c6b5d66638713446af3ed3e59e5cea87a6`의 #8727 플러그인은 `PluginRegistrar::registrar()`의 protected 접근(C2248)으로 실제 compile에 실패했다. `/W4 /WX`에서는 C4456/C4458 변수 가림도 실패했다.
- 공개 C 등록 진입점이 가진 `FlutterDesktopPluginRegistrarRef`에서 `FlutterDesktopRegistrarGetTextureRegistrar`로 texture registrar를 얻어 생성자에 전달했다. 소유권·thread·budget·공개 ABI를 바꾸지 않았다. 가려진 변수는 이름만 구분했다.
- 같은 `/utf-8 /W4 /WX /EHsc` 설정으로 plugin DLL과 공개 texture ABI·frame lifetime 실행 파일을 compile/link했다. Windows PE를 실제 실행해 ABI 검사와 lifetime 8 cases / 51 checks를 통과했다.
- MSBuild의 UNC 경로 대소문자 incremental 경고(MSB8064)와 임시 경로 경고(MSB8029)는 source compile 경고와 분리했다.
- 최초 실행 metadata의 `head`가 source가 아닌 runner cwd의 devel을 가리킨 것을 발견해 `runner_cwd_head`와 실제 `source_head`, 원 파일 SHA-256으로 정정했다. root 소스를 빌드했다고 주장하지 않는다.
- Native D3D Rust/backend와 Flutter의 실제 소비 pixel 검증은 이 C++ 성공만으로 완료 처리하지 않는다.

## 최소 글꼴 환경의 캐시 검사

집중 회귀는 193건 중 192건 통과·1건 실패했다. 실패는 P73 `cb59b901caf1e5a3c70c385f8b711424cbaaabc8`에서 추가한 `native_tiles_keep_page_and_exact_scale_keys_separate`다. 원 helper는 base와 같지만 새 검사가 두 페이지 crop의 픽셀 차이를 요구한다. 최소 글꼴 환경에서는 두 crop이 모두 흰색이고, 같은 source의 일반 host 글꼴 환경은 통과했다.

설치 글꼴 가정을 제거하고 기존 MIT 합성 `tests/fixtures/fonts/RHWPHostFixture-Regular.ttf`를 두 session에 명시했다. 픽셀 차이·uncached 동등성·page/scale key·LRU hit/miss/replay 기대값은 모두 유지했다. 보정 후 최소 글꼴 환경에서 해당 모듈 39건이 통과했다. 실물 조판의 golden·좌표·한컴 출력 기대값 변경은 없다. 기존 #8276/P71 → #8394 merge `7d4741392bb2ffaf308f5ce1cf6ba7f4deeeb1c9`의 검증과 이번 P73 신규 검사를 구분한다.

## 실제 Linux 실행과 남은 범위

- 실제 fresh Native GL library의 session·worker·snapshot/presentation·IME·저장·접근성·Qt CPU 검사가 통과했다. Qt GPU smoke는 21개 portable 검사와 실제 GL frame 9회·snapshot export 5회·전체 frame readback 0을 확인했다.
- 실제 GL·tile cache·context recovery와 Flutter 공개 SDK plugin compile/link·GL/GLES 상태 복원을 통과했다. 실행 backend는 Mesa llvmpipe 소프트웨어 GL이며 물리 GPU 성능 검증으로 승격하지 않는다.
- Flutter analyze 0 issue, 전체 Flutter test 70건 통과에 실제 Native IME 2건이 포함된다. `flutter pub get`이 만든 `analysis_options.yaml`의 build/platform 경로 제외 migration을 기록했다. 실제 C++ 검사는 별도로 실행했다.
- 실제 ABI direct PDF 53개 검사·clipboard·command·view·저장·접근성 검사를 통과했다. 최초 호출 인자 누락으로 종료된 진단과 올바른 인자로 재실행한 성공 결과는 JSON에서 구분한다.
- 최종 Flutter texture/controller view·Windows D3D runtime·macOS SDK·전체 공통 검증·fresh WASM·전쪽 시각 증적은 진행 전/진행 중이다. 이 중간 기록은 원 PR의 최종 수용 판정이 아니다.

정확한 실행 command·시각·source·산출물 해시는 [중간 실행 원장](host-validation-progress.json)을 따른다. 실행 로그와 SDK·도구·바이너리는 ignored `output/` 및 작업 전용 Windows 임시 경로에만 보관한다. 원격 push·댓글·새 PR·merge는 하지 않았다.

## 플랫폼 실행 후속 확인

[실제 플랫폼 실행 원장](platform-execution/summary.json)에 후속 결과를 추가했다. Windows Native D3D는 실제 SDK compile/link·실제 Windows Python의 session/worker/presentation/IME와 실제 하드웨어 D3D12의 색상·알파·좌표·stale 거부를 통과했다. 검증 전용 readback과 제품 경로의 readback 0은 구분한다. 최종 Windows Flutter D3D11 소비자 sampling과 지속 표시 qualification은 완료로 주장하지 않는다.

Linux Flutter controller/view의 실제 두 revision·가림·해제와 별도의 그림 입력 texture 캡처를 확인했다. CPU와 실제 화면의 같은 320×240 영역에서 내용 bbox가 같고, 2px 밖 잉크 차이 0이며 RGB의 가장자리 AA 차이는 남는다. 한컴 전쪽 시각 검증이나 물리 GPU 성능 근거로 전용하지 않는다.

편람의 첫 시도는 최소 글꼴 환경에서 Python fontTools 모듈을 숨겨 렌더 시작 전에 실패했다. Python 패키지 경로만 read-only로 보완하고 노출 글꼴 5개를 다시 확인한 다음 같은 source·기준·Native gate를 실행해 전 383쪽을 통과했다. 해당 TSV와 한도 하락 0을 기록했다.

#8337의 실제 입력에서는 원 수식 글꼴 부재 시 숫자가 다른 문자로 표시되는 P79 미완료 경계를 발견했다. 점수만으로 통과하지 않고 [실행 증거와 보정 계획](positioned-equation-fallback-finding.md)에 따라 보류했다. 다른 플랫폼 성공과 이 결함을 상쇄하지 않는다.
