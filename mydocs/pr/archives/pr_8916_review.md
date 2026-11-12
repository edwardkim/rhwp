# PR #8916 리뷰 — 명시적 0 크기 글자 문단의 줄 높이

## 최종 판정

**최종 재검증 중 — PR 제출 보류**. 최신 보정 source는 `d66568ef247859a9c184546cad3ba42b06e806e1`다. 이 PR의 같은 원본 1쪽은 새 Native **99.75%**, fresh WASM **99.72%**로 유지됐다. 실제 CanvasKit 두 모드의 크기 0 명령도 오류 없이 끝났다.

크기 0 문단과 U+F0FA 보정 후 실제 Studio 2입력 × 3백엔드 × screen/print 12캡처에서 런타임 계약 위반은 **0건**이다. 두 CanvasKit 모드가 실제 선택됐으며 숨은 Canvas2D overlay로 대신하지 않았다. 백엔드 간 report-only 픽셀 비교는 8건 중 4건이 문턱을 넘으므로 전체 픽셀 동등성을 주장하지 않는다. [추가 보정·집중 검증](../assets/hyunmyungjanelee_20261112/wingdings-arrow-shaft-repair/cause.md)과 [실제 소비 경로](../assets/hyunmyungjanelee_20261112/wingdings-arrow-shaft-repair/actual-consumers/summary.json)를 보존한다. 최종 공개 전쪽·Native/fresh WASM 정본 보호·전체 회귀·전체 실제 소비 경로 결과는 완료 후 갱신한다.

## 보정 전 전쪽 검증 기록 (8fe7d1fb)

**머지 보류** — 이 PR의 공개 입력 1종·1쪽은 Native 최저 99.75%, fresh WASM 최저 99.72%다. 두 경로 모두 90% 미만·devel 대비 2%p 초과 하락은 없으며 독립 Print와 쪽수가 일치한다. 이 이슈의 최소 글꼴 Native 집중 검사 1건도 모두 통과했다.

보정 전 검증 code head는 `8fe7d1fb8bc5024d0377ff7aa37d98c78e8f071a`다. 공통 Native 집중 126 PASS와 정본 Native 18종·1,771쪽의 공식 표시값 하락 0쪽을 확인했다. 최소 글꼴 전체 회귀는 **13,356 PASS / 0 FAIL / 45 skipped**(774.340초), fmt는 PASS다. [실제 전체 회귀 결과](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/full-nextest-validation.json)를 보존했다. Native·WASM·workspace all-target Clippy와 workspace build도 모두 PASS다. 정본 fresh WASM은 **18종·1,771쪽, 공식 표시값 하락·누락·추가 0쪽**으로 완료했다. [Native 보호](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/protected-native-validation.json)와 [fresh WASM 보호](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/protected-wasm-validation.json)를 각각 보존한다. 추가 Native Skia·suite/단위 정책도 모두 PASS다. [실제 게이트 기록](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/gates-validation.json)을 보존한다. 실제 Studio 48건의 캡처에서 #8916의 `textRun:invalidGeometry`와 #8918의 `textRun:glyphMapping`이 확인되어 소비 경로 검증은 미충족이다. [보정 전 증거](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/actual-backends-before/failure-summary.json)를 보존하고, 이 결함·지원 범위의 보정과 재검증이 필요하므로 최종 통합 판정은 그 완료 뒤 갱신한다. 과거 Cairo 점수나 이전 head 결과를 새 소스의 완료로 재인용하지 않는다.

## 접수와 검증 범위

- 원 PR [#8916](https://github.com/edwardkim/rhwp/pull/8916), 작성자 `hyunmyungJaneLee`, 이슈 [#8664](https://github.com/edwardkim/rhwp/issues/8664).
- 원 head `6ad97a9ae8bd230b1e2fd2d7a61764423e0c2b3b`·reviewer `jangster77`를 확인한 뒤 체리픽했다. 원 contributor 브랜치를 변경하지 않았다.
- devel 기준 `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`, 통합 브랜치 `review/integration-20261112-hyunmyungjanelee`. 원 29개 커밋의 [적용 기록](../assets/hyunmyungjanelee_20261112/cherry-pick-result.json)과 [additive 충돌 해소](../assets/hyunmyungjanelee_20261112/conflicts.json)를 보존한다.
- 현재 요청 reviewer는 `jangster77`다. 원격 원 head·devel은 검토 시작 때와 동일하다. 최종 push·merge 직전에 다시 확인한다.

## 변경과 조판 원칙 검토

모든 글자 모양의 크기가 0으로 선언된 문단과 크기 정보가 없는 문단을 구분한다. 명시적 0을 대체값 12px로 바꾸어 표를 아래로 미는 현상을 막는다.

`composer/line_breaking.rs::explicit_zero_size_paragraph`를 frame 구성과 재조판 저장 줄 생성이 함께 소비한다. 비율 줄간격 0 높이와 고정·최소·여백 줄간격의 본래 계약을 구분한다. 실제 TOP→표→AFTER 소속·순서와 일반 크기/미해결 스타일 대조군을 검증한다.

## 메인터너 보정과 수용 경계

MCP 2020은 원 PR의 같은 입력에 손상·변조 열기 경고를 내어 `HANCOM_DOCUMENT_OPEN_REJECTED`로 중단했다. 사용자가 제공하고 Print라고 확인한 한컴 2024 13.0.0.3901 PDF를 사용한다. win10-ted의 원본과 Git 입력 SHA도 일치한다. MCP 2020 성공으로 표시하지 않으며 이 수동 2024 기준을 기본 2020 엔진 쪽수 원장에 혼입하지 않는다.

## 공식 Print와 Native/fresh WASM 전쪽 대조

총 1종·1쪽, 최저 99.75%, 90% 미만 0쪽이다. 같은 입력·PDF의 devel 대비 2%p 초과 하락 0쪽이다. 기준 출력에 없던 쪽은 임의의 0점으로 채우지 않고 복원으로 구분한다. [Native 전쪽 비교 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/native-comparison.tsv)·[생산 SHA와 입력/PNG 해시](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/native-validation.json)를 연결한다.

| 입력 | 쪽수 | Native 최저 | fresh WASM |
| --- | ---: | ---: | --- |
| [zero_size_para_before_table.hwp](../../../samples/issue8664/zero_size_para_before_table.hwp) | 1 | 99.75% | 99.72% |

## 기준 PDF의 출처와 제외한 자료

원 PR이 기준으로 제시한 **한컴 웹한글기안기 PDF(Cairo)는 이 검토의 검증 대상에서 제외했다.** 동일 원본의 **Windows 데스크톱 한컴 인쇄(Print) PDF**로 대조했으며, 실제 출력 제품은 한컴 2024 13.0.0.3901의 사용자 제공 수동 Print 1종(MCP 2020 열기 거절과 구분)이다. 원본의 저장 제품과 Print 출력 제품을 혼동하지 않는다. [공식 Print 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)에 입력·PDF SHA-256, 엔진·제품 버전·Print method 0·작업 ID 또는 사용자 Print 확인을 보존했다.

현재 기준은 입력 34종 가운데 MCP Print 33종·사용자 확인 한컴 2024 Print 1종이다. 입력·기준 PDF는 Git에 보존하고, 이전 형식 미표기 경로에서 원본 형식·엔진을 명시한 정본 경로로 이름만 바로잡았다. [공식 Print 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)의 파일 SHA와 현재 실행·커밋이 일치한다. 이전 Cairo 기준은 Git 이력이며 현재 수용 근거로 쓰지 않는다.

## 증적과 직접 판독

[Native 직접 판독](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/native-direct-reading.json)과 [fresh WASM 직접 판독](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/wasm-direct-reading.json)에 대표 7쪽의 같은 페이지 review·overlay 확인을 기록했다. [fresh WASM 전쪽 비교](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/wasm-comparison.tsv)와 [실제 빌드·초기화 응답 해시](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/wasm-validation.json)를 함께 연결한다.

각 입력의 같은 쪽 raw·독립 PDF PNG·review·overlay 및 전체 TSV는 아래 폴더에 있다. 원 PR 주장에 대응하는 입력을 사용하며 다른 문서의 그림으로 대체하지 않는다.

- `pr8916-zero_size_para_before_table`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/overlay-001.png).

## 이 PR의 집중 회귀와 직접 판독

최소 글꼴 Native 실행에서 이슈 #8664에 대응하는 **1 PASS / 0 FAIL**을 확인했다. 공통 126건 중 해당 PR의 검사만 아래에 구분한다. 선택 밖 4,903건은 실행 완료 수에 넣지 않는다. [실제 명령·검사명·로그 해시](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/focused-validation.json)를 보존한다.

- `issue_8664_zero_size_paragraph_line_height::zero_size_paragraph_adds_no_line_before_inline_table`: PASS.

- `pr8916-zero_size_para_before_table` 1쪽: TOP→표→AFTER의 순서와 표의 높이·위치가 사용자 확인 2024 Print에 대응한다. MCP 2020 성공으로 표시하지 않는다.

아래 이미지는 이 PR의 같은 입력·같은 쪽이다. Native와 fresh WASM의 review·overlay를 각각 직접 열어 확인했으며 raw·PDF PNG도 함께 연결한다.

### `pr8916-zero_size_para_before_table` 1쪽

**native** · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/raw-001.png) · [독립 Print PNG](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/pdf-001.png)

![native review pr8916-zero_size_para_before_table p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/review-001.png)

![native overlay pr8916-zero_size_para_before_table p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/native/overlay-001.png)
**wasm** · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/wasm/raw-001.png) · [독립 Print PNG](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/wasm/pdf-001.png)

![wasm review pr8916-zero_size_para_before_table p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/wasm/review-001.png)

![wasm overlay pr8916-zero_size_para_before_table p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8916/pr8916-zero_size_para_before_table/wasm/overlay-001.png)

## 공통 최종 검증과 후속 처리

현재 Native 증적은 동결 실행 파일의 실제 생산 SHA와 현재 커밋 소스의 바이트 동일성을 구분해 기록했다. Native/fresh WASM 전쪽 판정, 최소 글꼴 전체 회귀, 필수 lint/build/Skia와 정본 보호 TSV의 공식 두 자리 하락 금지를 모두 충족한 후 통합 PR을 생성한다. 원 PR head·통합 head·CI·작업지시자 승인 및 원 이슈 전체 해결 여부를 구분하며, 원 PR은 통합 PR 병합을 확인한 뒤 증적을 연결해 후속 처리한다.
