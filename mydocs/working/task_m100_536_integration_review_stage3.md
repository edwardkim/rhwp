# seo-rii 통합 검토 3단계 — 실제 출력·ABI 선행 검증

## 검증한 후보와 실행 환경

코드 후보는 `456ac3585966116e38d4ee2491cb62439b5ebbd7`, 기준 devel은
`fe5725dc384689b45d60ef8ea4c83138bc2701d8`이다. Native와 WASM은 같은
`/home/tsjang/rhwp/target/pr-review`에서 Cargo 8 jobs로 순차 빌드했다.
원본·Print PDF·96 DPI·print profile·실제 한컴 글꼴 세 디렉터리를 기준과 후보에서 같게 유지했다.
실행 명령·입력/글꼴 해시·원본 TSV는 ignored
`output/pr-review/seo-rii-20261017/`의 실행 JSON과 각 `run_manifest.json`에 있다.
이 경로는 종료 전 정해진 영구 증거 경로로 옮기고 해시를 검증한다.

- Native `release-test/native-skia` 빌드 성공, 334.70초. 실행 파일 SHA-256
  `4f3e6b2874f2e2d1f7f8859bed9c05689c51b5a7522bb48f7112a383ae86b943`.
- 기본 Native Clippy와 `native-skia --lib` Clippy 모두 `-D warnings` 통과.
- fresh WASM 빌드 성공, 321.50초. Docker가 없는 호스트의 `--no-opt` 진단 빌드이며
  표준 Docker 최적화 빌드 완료로 표시하지 않는다. 패키지 파일별 해시는 `candidate-wasm-build.json`에 있다.
- 새 worktree의 `pkg`와 Studio public에 그 패키지를 공급한 뒤 TypeScript와 production build 통과.
  공급 전 TypeScript의 `@wasm/rhwp.js` 누락은 준비 오류였으며 공급 후 재실행으로 해소했다.
- frontend unit 2,310 PASS / 0 FAIL / 2 skipped, renderer contract guard 통과.

## 전쪽 Native / fresh WASM 비교

각 원본은 전체 1쪽이다. 아래는 2px 관용 내용 실루엣 일치율이며 대표 쪽만 선택한 실행이 아니다.

| 원본 | devel Native | 후보 Native | devel WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| lseg-01-basic.hwp | 99.74540 | 99.74540 | 99.93175 | 99.93175 |
| lseg-02-mixed.hwp | 99.80580 | 99.80580 | 99.84066 | 99.84066 |
| pic-crop-01.hwp | 99.98500 | 99.98500 | 99.74013 | 99.74013 |
| shape-group-02.hwp | 90.04403 | 100.00000 | 100.00000 | 100.00000 |
| equation-lim.hwp / 새 독립 Print | 88.42676 | 88.42676 | 89.77376 | 89.77376 |
| form-01.hwpx / 사용자 수동 Print | 99.88426 | 99.88426 | 99.87500 | 99.87500 |

이 범위에서 devel 대비 하락은 없지만 극한 수식이 90% 미만이므로 전체 시각 승인과 PR 준비는 미완료다.
Native shape의 source-center gradient와 외곽·그림자, lseg-01의 줄바꿈·문단 간격,
폼의 5개 개체와 콤보박스 문구를 review/standalone overlay에서 직접 대조했다.
폼의 기존 Edit 테두리와 글자 래스터 차이는 남아 있으며 이 PR이 폼 전체 외형을 새로 해결했다고 주장하지 않는다.
WASM 대표 패널은 같은 후보에서 이미 출력한 PNG를 canonical Visual Sweep 합성 함수로 만들고
원 캡처 manifest·PNG 해시를 `panel-provenance.json`에 연결했다. 재출력으로 표시하지 않는다.

## 사용자가 알려 준 기존 폼 PDF

`samples/hwpx/form-01-2024.pdf`는 [기존 #7578 기록](../pr/archives/pr_7578_review.md)의
사용자 수동 한컴 2024 Print다. SHA-256은
`d1117657d92c789295d73af1bebd11b241eb352254b328d95c07f0efb87b18ee`다.
사용자 요청에 따라 이 자료를 HWP/HWPX 폼의 공통 화면 대조에 사용했다.

HWP의 5개 개체와 HWPX의 주소·종류·크기·캡션·활성 상태·선택값은 대응한다.
HWPX는 위치 속성을 이름 있는 property로 추가 보존하고, ComboBox는 빈 선택값과 첫 목록
`계절 선택`을 분리 저장한다. HWP는 같은 표시 문구를 Text로 보존한다.
이 차이를 바이트 동일 또는 완전히 동일한 IR이라고 표시하지 않는다.
후보 Native의 두 형식은 실제 렌더 트리와 PNG 해시가 각각 동일했으며, HWP도 이 PDF와
Native 99.88426% / fresh WASM 99.87500%로 비교됐다.

이것은 HWP 자체를 새로 Print한 PDF가 아니다. 빈 `pdf/form-01-2022.pdf` 및 매크로로 거절된
MCP 작업 기록을 보존하고, 같은 화면의 대응 HWPX 수동 출력이라는 출처를 최종 review에도 명시한다.
빈 PDF에 맞춰 `Printable=1` 폼을 숨기거나 OnDocument_Open을 제거하지 않았다.

## 실제 Native / CanvasKit 계약

- `rhwp-native-ffi --features native-skia --profile release-test` 실제 공유 라이브러리 빌드 성공.
  파일 SHA·명령은 `candidate-ffi-enabled-build.json`에 보존했다.
- 그 라이브러리를 명시한 direct PDF, session, reference host, commands, file store,
  accessibility, view, worker, clipboard의 Python probe 9종 모두 exit 0.
  mock만 실행한 결과와 구분한다. PDF 기능 off·FFI lint·Rust panic test는 아직 별도 검증이 필요하다.
- 실제 CanvasKit의 글꼴 coverage 및 shaping replay 검사 통과.
- 실제 Chromium CanvasKit에서 정상 strict 2건, overflow 시 전체 fallback 18건 통과.
  첫 실행은 실행자가 없는 `/usr/bin/chromium`을 잘못 지정한 준비 오류였다.
  설치된 Chrome for Testing 154.0.8037.57을 명시해 재실행했고 제품 assertion을 실제 통과했다.
- 실제 Studio의 글꼴 준비 검사에서 Regular/Bold 두 로딩 순서, synthetic italic,
  screen/print 및 준비 실패 시 Canvas2D fallback과 진단을 확인했다.
- host 파일 I/O mock, file-store mock, Swift 포장 mock도 통과했다. 실제 Apple SDK·OS UI 및
  Linux Swift typecheck를 실행한 결과로 바꾸어 쓰지 않는다.

## 극한 수식 보정의 독립 근거와 다음 단계

원본 Print는 HyhwpEQ의 ASCII `lim`에 Tf 110, 본체 PUA에 Tf 92를 사용한다.
현재 producer는 이름을 추정 폭으로 측정하고 각 renderer가 일반 수학 글꼴로 다시 그린다.
그 결과 이름의 글꼴과 크기가 독립 Print와 다르다.

과거 [#280 조사](../report/archives/task_m100_280_report.md)는 일반 함수명 확대 규칙을
부정한다. 그 판단을 `sin/log` 전체의 확대 조건으로 뒤집지 않는다. 실제 HyhwpEQ ASCII와
PUA의 em 차이를 구분하기 위해 CLI로 11pt·16pt의 `lim`/`Lim` 대조 입력을 새로 만들고,
원본 전처리 없이 한컴 2020 Print를 각각 출력했다. 두 출력 모두 같은 HyhwpEQ face에서
이름/본문 Tf가 110/92, 160/133이며 이름 두 대소문자에 같은 보정을 사용한다.
입력 생성 명령·해시, Print job·제품·PrintMethod=0, PDF 해시와 실제 텍스트/font 연산은
`limit-probe-inputs.json`, `mcp-limit-*-status.json`, `limit-independent-print-font-observations.json`에 있다.
이 대조 입력은 글꼴 크기 진단용이며 아직 시각 통과한 정식 fixture로 추가하지 않았다.

수정 방향은 HyhwpEQ 경로에서 극한 이름을 공통 glyph/위치/글꼴 크기로 생산하고,
측정 → SVG/Native/Canvas2D/CanvasKit과 폰트 요청·preflight가 그 결과를 소비하게 하는 것이다.
다른 글꼴과 보호된 저장 상자 및 일반 EqLayout 경로는 별도 대조한다.
현재 실행 중인 27 case 집중 Rust 검사를 끝낸 뒤 이 보정과 관련 반례를 진행한다.
전체 nextest·최종 lint 묶음·Native Skia 3종·최종 fresh WASM·개별 review 확정·원격 PR/CI/merge는 남아 있다.
