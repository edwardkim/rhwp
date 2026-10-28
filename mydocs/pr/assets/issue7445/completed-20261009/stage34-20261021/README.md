# #7445 보정34 — KoPubWorld의 실제 공급 이름과 본문 frame 보존

## 원인과 기존 해결 계보

#8064의 온새미로 두 원본은 독립 Print46쪽과 내용 소유를 복구한 입력이다. 기존 #7642 stage2759·2856의 자동 줄/저장 흐름 보정, stage3508·3583의 본문 꼬리 원점과 기존 정식 검사를 먼저 대조했다. 이 브랜치의 stage29는 같은 SFNT의 KoPub 전체 이름/name6을 연결했지만 별도 KoPubWorld 프로그램은 포함하지 않았다.

최신 전체 Native 감사의 frozenV6는84/85조건에서47쪽 대 Print46쪽으로 실패했다. 기존 `issue_7642_confirmed_body_frame_origin`은 기본 환경에서는46쪽으로 통과했으나 실제 CLI의 `from_native_document_substitutions`를 적용하면 같은 c73d4c0dd9 라이브러리에서 두 형식 모두47쪽이다. `trueTypeFaces` 제거 또는 `preserveSourceMetrics` 해제도47쪽이므로 단순 메트릭 플래그 문제가 아니었다.

실제 CLI 공급 기록에서 원본 `KoPubWorld바탕체 Light`가 설치되어 있는데도 Haansoft Batang으로 선택됐다. Mac 설치 World 6개 SFNT의 name1/4는 굵기 이름을 포함하고 name6은 `KoPubWorldBatangLight` 등이며, fontdb의 typographic family 질의만으로 전체 이름의 원본 존재를 놓쳤다. 원본과 문서 지정 대체의 선택 순서가 뒤집힌 것이다. 측정값·쪽수·문서 ID·좌표를 맞추는 조건은 추가하지 않았다.

`document_requests → installed_font_program_aliases → Native SFNT 실제 조회 / 브라우저 공급 snapshot → from_document_lookup → 공통 FontEnvironment → 측정/paint`의 기존 경로에서 같은 World 파일의 이름만 연결했다. World와 KoPub, 다른 굵기, 별도 Pro 프로그램은 합치지 않는다. 원본 결측 시 문서 대체·명시적 환경·내장 프로그램 계약은 유지한다. 전179조건의 같은 공개 request API를 parser DocInfo로 대조해 요청이 달라지는 입력은 정확히84/85두 개였다. 이는 영향 분류이며 이전 코드 TSV를 새 head 검증으로 승격하는 근거가 아니다.

## 전쪽 실제 출력

원본 두 파일은 `samples/[2027] 온새미로 1 본교재.hwp`와 `.hwpx`다. 원본/PDF SHA는 같은 #8064 정본 원장84/85와 같으며 producer에서 재확인했다. Native는 Print/96DPI/compat2022·2024, fresh WASM도 각각 같은 호환과 PDFium153 smooth_rgb_v1/raw2px90%를 사용했다. 46쪽의 번호·누락·중복 없이 전쪽 TSV를 각각 새로 산출했다.

|형식|Native 최저|fresh WASM 최저|쪽수|90% 미만|
|---|---:|---:|---:|---:|
|HWP|91.43430%|94.45835%|각46/46|각0|
|HWPX|90.48362%|90.48362%|각46/46|각0|

[Native HWP](onsamiro-hwp-native.tsv) · [fresh HWP](onsamiro-hwp-fresh-wasm.tsv) · [Native HWPX](onsamiro-hwpx-native.tsv) · [fresh HWPX](onsamiro-hwpx-fresh-wasm.tsv) · [고정 base/후보/원래 하한 전쪽 대조](native-paired-pages.tsv).

고정80968497 Native도 같은 원본/PDF/실제 글꼴로 각각 전46쪽 새로 산출했다. base의 Native 최저는 HWP89.55513%,HWPX90.03945%다. base ABI에는 새 자동 공급 API가 없어 실제 기존 CLI와 같은 None 환경을 사용했으며, 후보는 실제 새 자동 공급 경로다. base를 공용 target에 재빌드하지 않고 출처/hash가 고정된 실행 파일을 사용했다.

Native HWP는 base 및 #8064 하한 대비0.1%p 이상 하락0쪽이다. **HWPX는 base 대비11쪽, #8064 하한 대비13쪽의 하락이 남는다.** 특히46쪽은95.62073→90.48362%(base대비-5.13711pp)이며 제목 글리프/자간 차이를 직접 판독했다. 이 단계의 쪽수·본문 복구로 잔여를 수용하거나 최종 PR 준비 완료로 처리하지 않는다.

## 회귀 검사와 중요 페이지

`issue_7445_installed_style_program`의 기존 API 검사에 World 동일 파일 이름을 보완했다. KoPub/World48조건과 다른 굵기·다른 세대·실제 결측의 반례를 검사하며, 미달 문서의 물리 px/golden을 추가한 검사가 아니다. 같은 최종 test source는 수정 전 실제1FAIL(18개 World 전체 이름 결측)→수정 후1PASS다.

기존 본문 frame 검사는 **None의 원래 두 경로를 그대로 유지**하고 결정적인 호출자 공급 snapshot 두 경로를 추가했다. OS의 개인 설치나 글꼴 바이너리에 의존하지 않는 프로그램 선택 입력이다. 실제 Native 공급/브라우저 공급은 위 전쪽 검증에서 별도로 확인했다. 같은 최종 source에서 수정 전47/46불일치로1FAIL→수정 후1PASS다. 기존 전46쪽,41쪽의 네 줄·원문 텍스트/순서/누락/중복·본문 내포와 독립 ±2CSSpx 기대값은 변경하지 않았다.

이 보완 전 Native/fresh 전46쪽은 모두90% 이상이며41쪽은 HWP99.61326/99.61330%,HWPX99.61303/99.61306%였다. 네 출력의 review와 standalone overlay를 직접 읽어 마지막 구절이 같은41쪽 본문 안에 한 번 있고 쪽 아래로 이월되지 않음을 확인했다. 글자 굵기·작은 잉크 차이는 남는다. raw99.61을 RGB 완전 일치로 표현하지 않는다.

정식 자동 suite 실행 결과는 아래 실행 기록과 working 보고서에 연결한다. 원래1개의 API test와1개의 본문 test를 보완했으며 test attribute 수를 늘리거나 기준선을 완화하지 않았다.

## review와 overlay

- 발견 당시 frozenV6 Native6쪽(47쪽 출력의 부분 실패 증거): [review](before-v6-native-p006-review.png) · [overlay](before-v6-native-p006-overlay.png). 이 PNG를 c73의 새 출력으로 표시하지 않는다.
- 후보 Native6쪽: [review](after-native-hwp-p006-review.png) · [overlay](after-native-hwp-p006-overlay.png).
- 후보 HWP41쪽: Native [review](after-native-hwp-p041-review.png) · [overlay](after-native-hwp-p041-overlay.png), fresh [review](after-fresh-hwp-p041-review.png) · [overlay](after-fresh-hwp-p041-overlay.png).
- 후보 HWPX41쪽: Native [review](after-native-hwpx-p041-review.png) · [overlay](after-native-hwpx-p041-overlay.png), fresh [review](after-fresh-hwpx-p041-review.png) · [overlay](after-fresh-hwpx-p041-overlay.png).
- 잔여 HWPX46쪽: base [review](base-native-hwpx-p046-review.png) · [overlay](base-native-hwpx-p046-overlay.png), 후보 [review](after-native-hwpx-p046-review.png) · [overlay](after-native-hwpx-p046-overlay.png).

## 실행 출처와 남은 범위

Native frozen CLI SHA `4f678c769dc5eb0261310aba5f4f8bace09c36cd7f9bc7fa21d9415d96964223`,lib `928a10c348e878c2dd913fb7ec33f4d641c8b2a59596abcc3d99077b0a95f33c`이며 jobs8/locked/native-skia release-test 빌드는187.076초 exit0이다. source는 parent c73d4c0dd9+이 단계 mod.rs diff/hash에 고정했다.

fresh는 locked wrapper web/pkg/no-opt의 Mac 로컬 대체 빌드242.436초 exit0이며 Docker 최적화 통과가 아니다. JS `d7d8cf9b5f7ce8a0f00b49a39aa1fee49e331cce6774e7ba57c3e9d078cd7030`,WASM `532d532299b90cbca2629345809c1b8e778ee8a025b40657cc09d249ab3eb6e7`가 루트 pkg/Studio public/동결 package와 일치한다. 새 브라우저의 실제 Studio7736 초기화 응답도 두 파일HTTP200/hash일치·wasmInitialized true/errors0이다. 문서 paint·CanvasKit·실제36건 완료로 쓰지 않는다.

정식 검사/전체 브랜치 gate는 부분 단계와 구분한다. 기존 frozenV6 전체 감사는177조건1990쪽 산출과84/85쪽수 불일치2조건을 남긴 이전 코드 결과이며 현재 source의179조건 통과로 합치지 않는다. 이 단계의 새로운92쪽 Native/92쪽 fresh, alias 영향2조건 및 실제 소비와 연결한다. HWPX 잔여·WMF·다른 완료 문서 하한·전체 nextest/필수 gate/실제 소비36건은 계속해야 한다. 원본/PDF/폰트·공유 target은 보존했다. 지침 변경·remote 변경은 없다.

로그·JSON·진단Rust·중간 PNG/SVG는 ignored `output/pr-review/7445-completed-20261009/rebase-80968497-20261021/kopubworld-style-program-v15/`에만 둔다. 완료 SVG 정리는 SHA manifest를 남겼으며 PNG/TSV/원본을 보존했다.

## 정식 자동 검사 결과

최종 source의 --prepare는1684 sources/7410 attrs/28 suites+20 exceptions=48 targets와 일치했다. 실제 배정은 설치 이름 API가 suite007, 본문 frame이 suite002다. 자동 nextest는 두 binary에서 **2실행/2PASS/0FAIL**, 실행0.484초·컴파일 포함213.705초 exit0이다.471개는 필터로 미선택된 검사이며 전체 회귀 skip/통과로 합산하지 않는다. 최종 body source SHA `d70ee297f6583d237c132f465f71cb29f30c5483a65d040a4c6ed538a94e3d09`, 프로그램 source SHA `2f2b7defc03c4cebefff903bdcf0bd78245efdc987cdcf5420476567530fedd8`가 수정 전후 SDK 증거와 정식 실행에 같다.

cargo fmt check, manifest check(base809), unit-tier check(base809), git diff check도 exit0이다. 단위 분류4101 tests/296modules, 기존7410 attrs를 유지한다. 이 단계에서 전체 nextest·전체 strict Clippy/Skia/실제36건 완료를 주장하지 않는다. 초기 SDK 진단의 box타입/지원하지 않는 base 자동API/출력 폴더 충돌과 DPI혼합 비교 오류는 ignored output에 구분했고 정식 성공 근거로 세지 않았다.
