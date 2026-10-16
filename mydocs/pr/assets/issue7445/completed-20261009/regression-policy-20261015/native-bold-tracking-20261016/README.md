# 굵은 제목의 자간 복원 (2026-10-16)

Issue: #7445. 기준 `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`,
부모 `fa16bec98d9c213419bf7dbd022799d74f4df330` 위의 C76 보정이다.

## 독립 근거와 원인

사용자가 지적한 `samples/issue_265.hwp`12쪽은 자연 글리프 폭은 맞고 글자 사이의 전진이 부족했다.
원본 문단121의 한글/영문 자간0·장평100은 유지한다. 독립 Print의 Type3 T2 `/Widths`에서
한글은1000/1000em=24CSSpx, S/c는15.552/11.64CSSpx다. 현재 자연 폭24/15.5733/11.6267과
가깝지만 PDF의TJ는 굵은 비공백 글자에 약0.05em 추가 전진을 넣는다. 그 결과 영문 시작점은
PDF331.68CSSpx인데 보정 전320.75CSSpx로 누적 차이가 났다. 글꼴 교체로 맞추지 않는다.

옛 프로그램의 출처는 HWP3 파서가 `Font.native_program`에 확정한다. 공통IR의 이 필드는
직렬화하지 않으며 표시 글꼴·원문 자간은 그대로다. 렌더러가 컨테이너 형식을 재판별하지 않는다.
기존 휴먼명조 프로그램 계약을 같은 출처로 옮기고 한양신명조의 굵은 비공백 전진을 분리했다.
`select_native_programs` → `char_width_decision` → 공통 줄/원점 → `TextRunNode`를 공유한다.
`glyph_fit_positions`는 추가 전진을 글리프 확대에 쓰지 않는다. SVG, web_canvas, Skia의 실제
소비에서 원점은 유지하고 자연 폭을 사용한다. 포맷을 렌더러에서 추정하던 C75 중간 후보는
커밋하지 않고 파서가 출처를 전달하는 C76으로 바꿨다.

## 전쪽과 정상 대조군

동일 입력·독립 PDF·공통 빈 FontEnvironment·96DPI/compat2022·2px 관용·캡처8 조건이다.

|검증|쪽수|Native 최저|fresh WASM 최저|90% 미만|
|---|---:|---:|---:|---:|
|원본 issue265|16/16|99.09329%|99.09641%|0|
|정상 HWP5 변환본|16/16|96.86495%|97.20070%|0|

원본12쪽 Native95.88260→99.65878%, WASM95.93029→99.65791%다.
변환본은 C74와 전16쪽 PNG/TSV가 각각 동일하고 원본의 C75→파서 출처 정규화 후에도
양쪽 전16쪽 PNG/TSV가 동일하다. 원본 제목/소제목·본문 줄바꿈·쪽말을 직접 판독했다.
정확한 고정 devel의 원본 최저54.47425/54.90799%, 변환본46.01010/46.27776%는 기존 미달이며
현재 브랜치에서 이미 보정한 문서를 제거하거나 잘못된 기대값으로 고정하는 근거로 쓰지 않는다.
전체48조건/전체 회귀 검증의 완료는 아니다. 변환본 WASM1쪽의 기존 기준 대비 감소는 별도 추적한다.

## 정식 회귀와 실제 소비

혼용 굵은 제목 후보12/13/15쪽 중 양쪽 최솟값99.65791%로 가장 높은12쪽을 선택했다.
독립 PDF의 한글145.76/영문331.68/닫는 괄호609.9208 및 기준선152.8을96CSSpx±2로 보호한다.
원문·자간0·쪽수16·글자별 단일 소유·순서/누락/중복·옆 run 연결·기준선 관계도 검사한다.
`native_bold_title_keeps_independent_origins_and_source_order`는 실제 수정 전1FAIL
(X143.2533 vs145.76,0.054초) → 수정 후1PASS(0.075초)다.
`native_bold_tracking_does_not_stretch_the_actual_glyph`는 실제 Native paint의 자연/추가 간격
글리프 픽셀을 대조해0차이를 확인했다(0.115초). 두 검사 합계2PASS이며 기존 SO-SUEOP3건,
파서·글꼴·paint 관련124건도 통과했다. 처음 원본의 비활성 char shape까지 bold라고 단정한
테스트 오류는 고친 뒤 동일 검사를 전후 재실행했다. 그 초기 실패는 원인 재현 증거에서 제외했다.

Native PNG/fresh WASM 전쪽 외에 같은12쪽의 SVG→browser raster를 양쪽에서 별도로 검증했다.
둘 다99.65453%이며 래스터 hash도 같다. review/overlay에서 제목·본문·로고의 보존을 판독했다.
실제 Studio 제품 파일 열기/CanvasView12쪽 screen·print 두 캡처는 모두99.48865%다.
처음 로드한JS/WASM 응답 hash를 root/pkg·Studio/public·frozen package와 대조했다.
영문 시작X331.547CSSpx이며 제목·소제목과 본문을 직접 판독했다. 제품 편집 가이드는 그대로이며
임의 마스킹하지 않았다. 전역 actual36 또는 지원 CanvasKit 수용을 뜻하지 않는다.

원본 SHA256 `645525c8cd5ec11b1742ba7cfc759f68622861916233b5e982385cdb12f0ced2`.
독립 PDF `pdf/issue_265-hwp-2020.pdf` SHA256
`c8e4218c7b80f1fb4f7877f5613eb6fd82fa1ca2b1ca3cde6266b81586357302`.
변환본 입력 SHA256 `dc2edf4737c110ab16a6c6543bceed9f5648d634a40e180ce3fc2725ec0f21d8`,
PDF SHA256 `fec78b1b1e8491b735247ff33ad38d3db69ff4cf1f55a6e22d4e81a289597f0b`.
Native binary SHA256 `a0fc96e2aee8ed73644eed03e4dc565f90647c36b61fef7689f9a49f6680d3dd`.
WASM SHA256 `b3826ba63fe6d34cad0c56d00cb15cc0aa13208a20af0d5e6cf91960b08e0533`.
생산 소스 차이 SHA256 `69ebb929b6f88984382d9fcd4cd80d795851ee2ea4545e923dbd7fc0dce75aee`.
Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.

TSV·명령·입력/글꼴/생산 hash·로그는 ignored output
`regression-policy-audit-20261015/rebase-0953598d-20261016/c76-*`,
`candidate-c76/`, `actual-title-c76*/`에 보존한다. 지침·사용자 초안은 변경하지 않았다.

## 비교 이미지

|출력|review|overlay|
|---|---|---|
|Native 수정 전|[review](before-native-review.png)|[overlay](before-native-overlay.png)|
|Native 수정 후|[review](after-native-review.png)|[overlay](after-native-overlay.png)|
|WASM 수정 전|[review](before-wasm-review.png)|[overlay](before-wasm-overlay.png)|
|WASM 수정 후|[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|
|Native SVG|[review](svg-native-review.png)|[overlay](svg-native-overlay.png)|
|WASM SVG|[review](svg-wasm-review.png)|[overlay](svg-wasm-overlay.png)|
|실제 Studio screen|[review](studio-screen-review.png)|[overlay](studio-screen-overlay.png)|
|실제 Studio print|[review](studio-print-review.png)|[overlay](studio-print-overlay.png)|
