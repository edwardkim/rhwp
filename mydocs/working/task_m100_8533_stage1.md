# #8533 Native 글꼴 공급과 편람 품질 게이트의 구현·검증

이 기록은 Native 자동 공급과 최신 검증 호스트의 로컬 검증 결과를 정리한다. 원격 PR·CI·병합은
별도 단계이며, #7445 전체 문서를 새로 보정하는 작업이 아니다.

## 구현 범위

- 일반 Native PNG·SVG·PDF 출력은 내장·설치된 프로그램을 우선 사용하고, 없는 글꼴은 공개
  공급 원장의 웹폰트로 보완한다. 알려지지 않은 이름의 Noto 일반 대체는 원본 공급 성공과
  구분하여 보고한다. 사용자 글꼴과 글꼴 바이너리를 저장소에 추가하지 않는다.
- `info --font-plan`은 언어 슬롯별 원본·문서 지정 대체·실제 공급 후보를 조회한다.
  [CLI 사용법](../manual/cli_commands.md)도 함께 갱신했다.
- 표시용 대체의 family가 달라져도 원본의 언어별 폭 표·공백·TTF/HFT 출처를 보존한다.
  실제 공급자가 원본 또는 문서 지정 프로그램을 선택한 경우에는 조회 이름과 프로그램 신원을
  확인하여 그 프로그램의 계약을 적용한다. 표시 치환과 원 프로그램 선택을 이름만으로 합치지 않는다.
- Native 공급 경로가 원본 TTF의 결측 기호에 표시용 대체의 자연 전진을 덮어쓰지 않게 하고,
  해당 기호의 실제 paint 위치도 원본이 소유한 칸과 일치시킨다.
- `getPagePaintFontRequests`는 해당 페이지에서 실제 사용하는 WMF·SVG·그림자·점 리더의
  글꼴만 요청한다. 64MiB 메타파일 글꼴 한도와 순회·글리프 수 한도를 높이지 않는다.
- WASM Sweep의 암호 문서는 원본 바이트를 `openWithPassword`로 열고 암호를 stdin으로만
  전달한다. 다른 문서로 치환하거나 명령 인자에 암호를 넣지 않는다.
- 일반 Sweep에서 편람을 자동 실행하지 않는다. 조판 영향 PR의 제출 전에는
  `--handbook-gate`로 Native 383쪽을 별도로 실행하여 기존 페이지별 하한을 확인한다.

세션 환경과 실제 프로그램 선택의 차이는 [글꼴 환경 설명](../manual/font_environment.md)에 있다.
CI 실행 정의·판정 정책을 수정한 PR의 후속 기록을 옵션 2로 분리하는 지침과, 원 PR의 주장을
입증하는 페이지의 raw PNG를 review 코멘트에 첨부하는 지침도 사용자 지시에 따라 갱신했다.

## 생산 코드와 입력

검증 Native·WASM 모듈의 Rust 소스는 `c031a38a41670f5e4f6ab0268ef46c62ab1445f5`다.
이후 Node lifecycle 검사·문서와, [2단계 명시 환경](task_m100_8533_stage2.md)·
[3단계 PostScript 별칭](task_m100_8533_stage3.md) 호스트 보정을 구분한다. 최신 호스트는 `3ededa625581f4a4a052ef912dc30c0317fec0ed`다. 모듈 자체와 명시 환경을 사용하는 JavaScript 소비 경계의 검증 범위를 섞지 않는다.

| 항목 | SHA-256 또는 식별자 |
| --- | --- |
| Native Skia 고정 실행 파일 | `c2a5651cf29dc68f7b60d20395ca06f6b6d4eccf020db802d3188a3f1d4a280c` |
| fresh WASM | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| root pkg·Studio public JS | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` |
| 비교 base | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 편람 HWPX 입력 | `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6` |
| 독립 한컴 Print PDF | `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e` |

입력은 [편람 HWPX](<../../samples/2025 행정업무운영 편람(최종).hwpx>), 기준은
[기존 독립 Print PDF](<../../pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf>)를 재사용한다.
PDFium·96 DPI·2024 호환·Print·2px 실루엣 조건을 유지한다. Mac WASM은 locked wrapper의
`--target web --out-dir pkg --no-opt` 로컬 빌드이며 Docker 최적화 빌드로 기록하지 않는다.

## 확인한 결과

| 범위 | 확인 결과 |
| --- | --- |
| 전체 release-test nextest | 12,424 PASS, 0 FAIL, 45 skip; 실제 테스트 실행 581.283초 |
| release build·lib | build PASS; root 3,801 PASS·12 ignored, 다른 workspace lib 182 PASS |
| Native Skia workspace lib | root 3,857 PASS·12 ignored, 다른 workspace lib 합계 248 PASS |
| 정본 그림 placeholder | 2 PASS |
| 직접 PDF 출력 | 22 PASS |
| 추가 inline placeholder·독립 Print 기관명 상자 | 각각 1 PASS |
| Rust lint·build | fmt, Native/WASM Clippy, workspace build·all-target Clippy PASS |
| 문서 예제 | 8 PASS, 3 ignored |
| Studio | TypeScript PASS, Node 2,717 PASS·2 skip |
| Visual Sweep Python 계약 | 240 PASS |
| 현재 글꼴 환경·공급·원본/암호 Node 계약 | 34 PASS |
| 원 PR 입력 Native | 36문서·1,201쪽 완료, 같은 환경의 원 PR 재출력과 모든 PNG hash 동일 |
| #8438 입력의 최신 devel Native 추가 대조 | 10문서·78쪽, devel 동등 producer와 후보 PNG hash 동일 |
| 최신 호스트의 원 PR WASM 전쪽 대조 | 36문서·1,201쪽 모두 base/후보 PNG hash 동일; 순수 base 35문서와 읽기 전용 조회 API base 1문서를 구분 |
| 추가 글꼴 파일 접근을 차단한 Native 편람 | 383쪽 완료, raw 최저 99.96465%·130쪽, 기존 표시 하한 대비 하락 0쪽 |
| 같은 현재 환경의 base·후보 Native 편람 | 설치 환경 최저 94.26497→99.96463%, 차단 환경 93.13408→99.96465%; 각 383쪽, 0.1%p 초과 하락 0쪽 |
| 최신 호스트의 일반 WASM 편람 | base/후보 383쪽 PNG hash 동일; raw 최저 99.68453%·198쪽 |
| Native 공개 프로그램을 명시한 추가 WASM | 383쪽·최저 94.37243%·220쪽, 90% 미달 0쪽; Native·일반 WASM 환경과 구분 |
| 실제 일반 Native CLI | 전 383쪽 단일 export, 온라인·오프라인 집중 4쪽 PNG hash 동일 |
| Native SVG·PDF 실제 소비 | 272쪽 SVG의 Chrome 출력 99.77168%, 두 PDF backend 각각 100% |

전체 nextest에서 처음 보고된 글꼴 원장 검사와 중첩 표 검사는 모두 통과했다. 검사를 지우거나
좌표 허용치를 완화하여 실패를 없애지 않았다. 생성 harness 5개의 shard 배정 drift는 별도로
정정했다. 정정 전후 28개 생성 harness의 case source 집합 1,970개가 동일하고 중복이 없음을 확인했으며,
manifest·fmt·unit tier를 다시 통과했다. 원래 실패 로그는 통과 결과로 덮어쓰지 않는다.

완료한 TSV 원본·정밀 raw 값·PNG 해시와 선별한 실제 raw·review·overlay는
[제출 증적 묶음](../pr/assets/issue8533/final-native-webfonts/README.md)에 보존한다.
제품·검증 호스트·일반/명시 글꼴 환경을 구분하며, 임시 driver·로그·글꼴 프로그램을 포함하지 않는다.

글꼴 lifecycle 감사는 과거 고정 개수를 현재 개수로 바꾸는 대신, 봉인된 v1과 기록된 change set의
추가·퇴역·교체를 재생하여 기대 집합을 도출한다. 현재 v2에 기록된 18개 change set의 재생과
projection 일치도 별도로 확인했다. 이전 마이그레이션 초기 상태만 검사하는 W1/W7 진단은
현행 검증과 구분하며 봉인된 원장이나 초기 baseline을 재생성하지 않는다.

## 브라우저 검사와 환경의 한계

개인·공유 글꼴 파일을 읽지 못하게 한 Mac 프로세스 격리는 깨끗한 macOS VM과 같다고 주장하지
않는다. 이 상태의 Studio는 CoreText에 남은 이름을 설치 글꼴로 감지하면서 실제 윤곽을 읽지
못해 글리프 누락이 발생했다. 이 실행을 정상 Studio 또는 최소 글꼴 환경의 통과 증거로 쓰지 않는다.

설치 글꼴을 보존하고 Local Font Access 권한을 실제로 거부한 별도 Chrome에서 편람272쪽,
기관명5쪽, HFT 원본4쪽, 언어 슬롯 대조군1쪽의 Canvas2D 화면을 확인했다. 실제 CanvasView의
해당 페이지 사각형을 캡처했으며 flow canvas의 데이터만 추출한 것이 아니다. 이 네 문서의
CanvasKit 요청은 글꼴 준비 제약으로 Canvas2D에 fallback했으므로 CanvasKit 통과로 세지 않는다.

지원 대조군 `group-drawing-02.hwp`는 실제 CanvasKit software로 완료했다. readiness gate가
참이고 미지원 연산·미등록 글꼴 대체·숨은 Canvas2D overlay가 없었다. resource guard는 유지했다.

초기화에 실제 사용한 WASM 응답 hash는 root pkg·Studio public과 같다. Vite가 JS의 WASM URL
한 곳을 `/@fs/…`로 바꾸므로 수신 JS 자체의 hash가 root 파일과 같다고 주장하지 않는다. 실제
수신 응답의 sourcemap 원문에서 해당 URL 변환을 되돌리면 root JS hash와 일치한다.

## 시각 판정과 제출 경계

- 최신 호스트에서 원 PR 36문서·1,201쪽과 편람 383쪽을 다시 산출했다. 총 1,584쪽의
  base/후보 PNG hash가 동일하며 90% 미달·0.1%p 초과 하락은 없다. 35문서는 순수 base다.
  사양서 저장본 1문서는 순수 base의 64MiB 상한 실패를 보존하고 읽기 전용 쪽 자원 조회 API만
  더한 별도 producer를 사용했다. 순수/관측 대조 3쪽도 PNG가 동일하며 순수 base와 구분한다.
- Native 편람은 설치·파일 접근 차단 환경 모두 382쪽의 소수 둘째 자리 표시값이 100%,
  기존 예외인 물리 130쪽이 99.96%다. raw 자체가 100%인 쪽과 표시값 100%를 구분한다.
- 일반 WASM 최저 물리 198쪽의 하단 숫자·글리프 차이는 같은 base에서도 재현된다.
  전쪽 PNG 동일성으로 새 코드 회귀와 구분하며 Native와 같은 수치라고 주장하지 않는다.
- 추가 명시 환경 WASM은 Native 자동 공급 영수증의 공개 프로그램 9개와 저장소 공개 번들을
  전달했다. 전 383쪽이 90% 이상이고 272쪽은 100%지만, 220쪽의 글머리표 뒤 간격은 남아 있다.
  Canvas 보충 측정이 portable 6.666667px 대신 12.930099px를 소비하는 진단과 일치한다.
  이 결과를 일반 WASM 자동 다운로드 기능이나 Native 100% 결과로 표시하지 않는다.
- 기관명 문서 5쪽의 고정 px 검사는 Native 98.91953%·현재 WASM 99.37969%에서 독립 PDF의
  원점·폭 ±2px를 보호한다. raw 91.74074%인 같은 문서 3쪽으로 이 조건을 대신하지 않는다.
- 원 PR 주장의 해당 물리 쪽을 raw·review·overlay로 연결하고 글리프·간격의 잔여도 기록한다.
  PR 직전 base/head·입력/소스 동일성을 재확인하며, 채번·정확한 code head CI 이후 검토·오늘할일을
  반영한다. 병합·후속 처리는 [정본 종료 절차](../manual/pr_review/post_merge.md)를 따른다.

로그·실행 중 상태 JSON·임시 driver와 글꼴 프로그램은 ignored output에만 보존한다. 제출 시에는
최종 TSV와 선별한 PNG, 이 기록에서 확인한 출처·명령·판정을 정식 증적에 연결한다.
