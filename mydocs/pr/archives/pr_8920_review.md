# PR #8920 리뷰 — 쪽에 걸친 표의 저장 줄 없는 칸 직선 소유

## 최종 판정

**최종 재검증 중 — PR 제출 보류**. 최신 보정 source는 `d66568ef247859a9c184546cad3ba42b06e806e1`다. 아래의 이 PR 전쪽 수치는 보정 전 8fe7d1fb 결과다. 새 소스의 공개 전쪽·정본 보호·전체 회귀를 갱신 중이며, 이전 head의 통과를 최신 head 통과로 바꾸어 인용하지 않는다.

크기 0 문단과 U+F0FA 보정 후 실제 Studio 2입력 × 3백엔드 × screen/print 12캡처에서 런타임 계약 위반은 **0건**이다. 두 CanvasKit 모드가 실제 선택됐으며 숨은 Canvas2D overlay로 대신하지 않았다. 백엔드 간 report-only 픽셀 비교는 8건 중 4건이 문턱을 넘으므로 전체 픽셀 동등성을 주장하지 않는다. [추가 보정·집중 검증](../assets/hyunmyungjanelee_20261112/wingdings-arrow-shaft-repair/cause.md)과 [실제 소비 경로](../assets/hyunmyungjanelee_20261112/wingdings-arrow-shaft-repair/actual-consumers/summary.json)를 보존한다. 최종 공개 전쪽·Native/fresh WASM 정본 보호·전체 회귀·전체 실제 소비 경로 결과는 완료 후 갱신한다.

## 보정 전 전쪽 검증 기록 (8fe7d1fb)

**머지 보류** — 이 PR의 공개 입력 4종·8쪽은 Native 최저 100.00%, fresh WASM 최저 100.00%다. 두 경로 모두 90% 미만·devel 대비 2%p 초과 하락은 없으며 독립 Print와 쪽수가 일치한다. 이 이슈의 최소 글꼴 Native 집중 검사 5건도 모두 통과했다.

보정 전 검증 code head는 `8fe7d1fb8bc5024d0377ff7aa37d98c78e8f071a`다. 공통 Native 집중 126 PASS와 정본 Native 18종·1,771쪽의 공식 표시값 하락 0쪽을 확인했다. 최소 글꼴 전체 회귀는 **13,356 PASS / 0 FAIL / 45 skipped**(774.340초), fmt는 PASS다. [실제 전체 회귀 결과](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/full-nextest-validation.json)를 보존했다. Native·WASM·workspace all-target Clippy와 workspace build도 모두 PASS다. 정본 fresh WASM은 **18종·1,771쪽, 공식 표시값 하락·누락·추가 0쪽**으로 완료했다. [Native 보호](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/protected-native-validation.json)와 [fresh WASM 보호](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/protected-wasm-validation.json)를 각각 보존한다. 추가 Native Skia·suite/단위 정책도 모두 PASS다. [실제 게이트 기록](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/gates-validation.json)을 보존한다. 실제 Studio 48건의 캡처에서 #8916의 `textRun:invalidGeometry`와 #8918의 `textRun:glyphMapping`이 확인되어 소비 경로 검증은 미충족이다. [보정 전 증거](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/actual-backends-before/failure-summary.json)를 보존하고, 이 결함·지원 범위의 보정과 재검증이 필요하므로 최종 통합 판정은 그 완료 뒤 갱신한다. 과거 Cairo 점수나 이전 head 결과를 새 소스의 완료로 재인용하지 않는다.

## 접수와 검증 범위

- 원 PR [#8920](https://github.com/edwardkim/rhwp/pull/8920), 작성자 `hyunmyungJaneLee`, 이슈 [#8690](https://github.com/edwardkim/rhwp/issues/8690).
- 원 head `5037330c850e041591cbee1745120e98cf95e12c`·reviewer `jangster77`를 확인한 뒤 체리픽했다. 원 contributor 브랜치를 변경하지 않았다.
- devel 기준 `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`, 통합 브랜치 `review/integration-20261112-hyunmyungjanelee`. 원 29개 커밋의 [적용 기록](../assets/hyunmyungjanelee_20261112/cherry-pick-result.json)과 [additive 충돌 해소](../assets/hyunmyungjanelee_20261112/conflicts.json)를 보존한다.
- 현재 요청 reviewer는 `jangster77`다. 원격 원 head·devel은 검토 시작 때와 동일하다. 최종 push·merge 직전에 다시 확인한다.

## 변경과 조판 원칙 검토

줄 수 0인 non-TAC 직선 문단을 무조건 건너뛰지 않고 온전한 행·칸 전체 컷의 소유를 보존한다. 이미 나뉜 부분 컷에 같은 도형을 중복 방출하지 않는다.

`layout/table_partial.rs`에서 `whole_cell_cut`·`whole_cell_shape_paragraph`로 시작·끝 컷과 도형 소유를 확인한다. 빈 문단의 정렬 높이는 줄 띠와 개체 점유 끝 중 큰 값으로 측정하며 같은 소유 판정을 paint 문단 순회가 사용한다. 4종 8쪽의 위/가운데 정렬과 rowspan·글앞 도형 대조군을 확인한다.

## 메인터너 보정과 수용 경계

원 PR의 non-TAC 직선 소유 복구에 더해, 새 MCP PDF에서 확인한 중앙 정렬 잔여를 `3f27a1d9ee`로 보정했다. 저장 줄 없는 도형 전용 문단의 원본 em과 개체 범위 중 큰 띠를 측정·배치가 공유하며, `TopAndBottom`이 시각 bottom에서 빠져도 글자 줄 상자는 남는다. 새 검사는 고정 PDF 좌표가 아니라 원본 글자 크기·여백·오프셋과 실제 칸 상자의 관계를 대조한다. 일반/병합·위/가운데 4종 Native 전 8쪽은 100%다.

## 공식 Print와 Native/fresh WASM 전쪽 대조

총 4종·8쪽, 최저 100.00%, 90% 미만 0쪽이다. 같은 입력·PDF의 devel 대비 2%p 초과 하락 0쪽이다. 기준 출력에 없던 쪽은 임의의 0점으로 채우지 않고 복원으로 구분한다. [Native 전쪽 비교 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/native-comparison.tsv)·[생산 SHA와 입력/PNG 해시](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/native-validation.json)를 연결한다.

| 입력 | 쪽수 | Native 최저 | fresh WASM |
| --- | ---: | ---: | --- |
| [float_line_rowspan_infront_center.hwp](../../../samples/issue8690/float_line_rowspan_infront_center.hwp) | 2 | 100.00% | 100.00% |
| [float_line_rowspan_infront_top.hwp](../../../samples/issue8690/float_line_rowspan_infront_top.hwp) | 2 | 100.00% | 100.00% |
| [float_line_valign_center.hwp](../../../samples/issue8690/float_line_valign_center.hwp) | 2 | 100.00% | 100.00% |
| [float_line_valign_top.hwp](../../../samples/issue8690/float_line_valign_top.hwp) | 2 | 100.00% | 100.00% |

## 기준 PDF의 출처와 제외한 자료

원 PR이 기준으로 제시한 **한컴 웹한글기안기 PDF(Cairo)는 이 검토의 검증 대상에서 제외했다.** 동일 원본의 **Windows 데스크톱 한컴 인쇄(Print) PDF**로 대조했으며, 실제 출력 제품은 한컴 2020 11.0.0.9136(MCP `engine: 2020`, HOffice110) 4종이다. 원본의 저장 제품과 Print 출력 제품을 혼동하지 않는다. [공식 Print 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)에 입력·PDF SHA-256, 엔진·제품 버전·Print method 0·작업 ID 또는 사용자 Print 확인을 보존했다.

현재 기준은 입력 34종 가운데 MCP Print 33종·사용자 확인 한컴 2024 Print 1종이다. 입력·기준 PDF는 Git에 보존하고, 이전 형식 미표기 경로에서 원본 형식·엔진을 명시한 정본 경로로 이름만 바로잡았다. [공식 Print 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)의 파일 SHA와 현재 실행·커밋이 일치한다. 이전 Cairo 기준은 Git 이력이며 현재 수용 근거로 쓰지 않는다.

## 증적과 직접 판독

[Native 직접 판독](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/native-direct-reading.json)과 [fresh WASM 직접 판독](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/wasm-direct-reading.json)에 대표 7쪽의 같은 페이지 review·overlay 확인을 기록했다. [fresh WASM 전쪽 비교](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/wasm-comparison.tsv)와 [실제 빌드·초기화 응답 해시](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/wasm-validation.json)를 함께 연결한다.

각 입력의 같은 쪽 raw·독립 PDF PNG·review·overlay 및 전체 TSV는 아래 폴더에 있다. 원 PR 주장에 대응하는 입력을 사용하며 다른 문서의 그림으로 대체하지 않는다.

- `pr8920-float_line_rowspan_infront_center`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/overlay-001.png).
- `pr8920-float_line_rowspan_infront_top`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_top/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_top/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_top/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_top/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_top/native/overlay-001.png).
- `pr8920-float_line_valign_center`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_center/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_center/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_center/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_center/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_center/native/overlay-001.png).
- `pr8920-float_line_valign_top`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_top/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_top/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_top/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_top/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_valign_top/native/overlay-001.png).

## 이 PR의 집중 회귀와 직접 판독

최소 글꼴 Native 실행에서 이슈 #8690에 대응하는 **5 PASS / 0 FAIL**을 확인했다. 공통 126건 중 해당 PR의 검사만 아래에 구분한다. 선택 밖 4,903건은 실행 완료 수에 넣지 않는다. [실제 명령·검사명·로그 해시](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/focused-validation.json)를 보존한다.

- `issue_8690_partial_table_float_line_cell::float_line_cell_paragraph_emits_line_in_every_whole_row_center_aligned`: PASS.
- `issue_8690_partial_table_float_line_cell::centered_shape_anchor_keeps_the_source_font_line_box`: PASS.
- `issue_8690_partial_table_float_line_cell::float_line_cell_paragraph_emits_line_in_every_whole_row_top_aligned`: PASS.
- `issue_8690_partial_table_float_line_cell::infront_line_in_rowspan_cell_is_emitted_center_aligned`: PASS.
- `issue_8690_partial_table_float_line_cell::infront_line_in_rowspan_cell_is_emitted_top_aligned`: PASS.

- `pr8920-float_line_rowspan_infront_center` 1쪽: 병합 칸과 일반 칸의 직선이 모두 표시되고 가운데 정렬 위치가 Print와 대응한다. fresh WASM 100.00%.

아래 이미지는 이 PR의 같은 입력·같은 쪽이다. Native와 fresh WASM의 review·overlay를 각각 직접 열어 확인했으며 raw·PDF PNG도 함께 연결한다.

### `pr8920-float_line_rowspan_infront_center` 1쪽

**native** · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/raw-001.png) · [독립 Print PNG](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/pdf-001.png)

![native review pr8920-float_line_rowspan_infront_center p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/review-001.png)

![native overlay pr8920-float_line_rowspan_infront_center p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/native/overlay-001.png)
**wasm** · [raw](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/wasm/raw-001.png) · [독립 Print PNG](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/wasm/pdf-001.png)

![wasm review pr8920-float_line_rowspan_infront_center p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/wasm/review-001.png)

![wasm overlay pr8920-float_line_rowspan_infront_center p001](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/pr8920/pr8920-float_line_rowspan_infront_center/wasm/overlay-001.png)

## 공통 최종 검증과 후속 처리

현재 Native 증적은 동결 실행 파일의 실제 생산 SHA와 현재 커밋 소스의 바이트 동일성을 구분해 기록했다. Native/fresh WASM 전쪽 판정, 최소 글꼴 전체 회귀, 필수 lint/build/Skia와 정본 보호 TSV의 공식 두 자리 하락 금지를 모두 충족한 후 통합 PR을 생성한다. 원 PR head·통합 head·CI·작업지시자 승인 및 원 이슈 전체 해결 여부를 구분하며, 원 PR은 통합 PR 병합을 확인한 뒤 증적을 연결해 후속 처리한다.
