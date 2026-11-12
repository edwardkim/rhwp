# PR #8900 리뷰 — 양수 차트 값축 시작값·눈금 간격

## 최종 판정

**메인터너 보정 후 통합 가능 — 통합 PR의 exact-head CI 확인 필요**. 현재 검증 제품 source는 `73d06ed0b18df7c222a850bb5d9340c9b7de2bcc`, base는 `a85d3265dd10b316d1a0ae029ca3f67245b38b77`(#9017)다. 최소 글꼴 Native 집중 **164 PASS / 0 FAIL**, 전체 nextest **13,380 PASS / 0 FAIL / 45 skipped**(실행 404.945초)와 필수 게이트를 완료했다. [최신 실제 실행 원장](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/rebase-a85d3265/gates-validation.json)에 명령·종료 코드·로그 해시를 보존했다. 원 PR head를 그대로 승인하는 판정이 아니라 아래 메인터너 보정을 포함한 통합 후보의 판정이다.

시각 증적을 실제 산출한 source는 `30fa7739f0340f2a248b22542c7dd36d9354f056`, 당시 base는 `1a3ed4a7cd76701085a24ac81d30bc5fb31942c1`(#8956)이다. 이 PR의 공개 입력 4종·4쪽을 새 Native/fresh WASM으로 다시 출력해 최저 **95.91% / 95.08%**, 미달·새 하락 0쪽을 확인했다. 공개 전체 34종·46쪽의 raw·PDF·review·overlay 368개는 리베이스 전 직접 판독한 증적과 바이트가 같다.

#9017 리베이스의 44개 patch ID와 기존 출력·입력·폰트·캡처 경로 동일성을 확인해 전쪽 TSV·PNG·독립 Print를 재사용했다. 원 producer와 원 source를 유지하며 새 head에서 전쪽을 재캡처했다고 표시하지 않는다. 같은 최소 글꼴 조건의 새 base/후보 HTML 계약 23건은 모두 통과했고, 최신 WASM의 실제 브라우저 본문·셀 붙여넣기 6사례 및 HWP/HWPX 두 차례 저장 24회, 기존 표 경로 API 11건도 통과했다. [재사용 경계](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/rebase-a85d3265/README.md), [새 base 대조](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/rebase-a85d3265/incoming-base-validation.json), [실제 응답 해시](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/rebase-a85d3265/browser-runtime-result.json)를 구분한다.

시각 증적 source `30fa7739`의 최소 글꼴 Native 집중 **141 PASS / 0 FAIL**에 이 PR의 4건과 새 #8956 API 4건을 포함했다. 실제 브라우저 표 경로 API 11건도 PASS이며 Root pkg·Studio public·초기화 응답의 JS/WASM 해시를 대조했다. 실제 Studio 48개 강제 진단 캡처의 현재 hard-gate 위반은 0건이지만 이 값만으로 내용 보존을 판정하지 않았다. CanvasKit 지원 캡처 28개와, 기존 미지원 RawSvg를 강제한 차트 4개를 구분했다. 차트의 정상 auto 선택은 사전 검사로 Canvas2D에 전환하며 양 모드 screen/print 4회에서 차트를 보존했다(Print 95.08%). 미지원 4회를 CanvasKit 지원 통과로 세지 않는다. 백엔드 간 report-only 픽셀 비교는 32건 중 16건 미달이며 픽셀 동등성을 주장하지 않는다. [현재 실제 소비 결과](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/actual-backends/summary.json)를 연결한다.

편람 Native 383쪽 gate는 최저 99.96%·공식 표시값 하락 0쪽으로 통과했다. 정본 Native/fresh WASM 각 18종·1771쪽의 TSV도 현재 canonical로 다시 계산해 최저 98.36%·공식 표시값 하락 0쪽을 확인했다. 정본 PNG의 원 producer d66568ef와 `30fa7739`의 직접 편람 캡처를 구분한다. [리베이스 소비 경로·재사용 근거](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/rebase-read-path-audit.md), [Native 보호](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/protected-native-validation.json), [WASM 보호](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/protected-wasm-validation.json)를 보존한다. `30fa7739`에서는 최소 글꼴 전체 회귀 13,365 PASS / 0 FAIL / 45 skipped와 필수 16단계 게이트를 모두 통과했고, #8956의 새 base Native 4건·후보 4건·브라우저 API 11건도 통과했다. [전체 실행 결과](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/gates-validation.json)와 [base/후보 대조](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/incoming-base-validation.json)를 현재 #9017 추가 검증과 구분한다.

초기 source 8fe7d1fb에서는 크기 0·F0FA의 CanvasKit 결함이 발견됐고, 이후 이를 보정했다. 초기 전체 회귀나 초기 Cairo/웹한글기안기 점수를 현재 head의 완료 근거로 사용하지 않는다. [이전 결함 기록](../assets/hyunmyungjanelee_20261112/final-print-8fe7d1fb/actual-backends-before/failure-summary.json)은 이력으로 보존한다.

## 접수와 검증 범위

- 원 PR [#8900](https://github.com/edwardkim/rhwp/pull/8900), 작성자 `hyunmyungJaneLee`, 이슈 [#8678](https://github.com/edwardkim/rhwp/issues/8678).
- 원 head `b215fce8179da1b5ed1a3c3c889388611a87c093`·reviewer `edwardkim`를 확인한 뒤 체리픽했다. 원 contributor 브랜치를 변경하지 않았다.
- 최초 devel 기준 `80b1c354c29b7a02fb2b3c14a274da7f5b653ada`(#8956·#9017 반영 뒤 현재 base는 위 a85d3265), 통합 브랜치 `review/integration-20261112-hyunmyungjanelee`. 원 29개 커밋의 [적용 기록](../assets/hyunmyungjanelee_20261112/cherry-pick-result.json)과 [additive 충돌 해소](../assets/hyunmyungjanelee_20261112/conflicts.json)를 보존한다.
- 현재 요청 reviewer는 `edwardkim, jangster77`다. 원 PR head의 출처를 보존했다. devel은 #8956·#9017 반영으로 갱신됐으며, 최종 push·merge 직전에 원 PR head와 devel을 다시 확인한다.

## 변경과 조판 원칙 검토

최솟값 부근에 모인 양수 차트의 자동 축 floor와 step을 독립 한컴 차트 출력의 관측 규칙과 대조한다. 차트 전체 피델리티나 관련 이슈 전체 해결을 주장하지 않는다.

`crates/rhwp-ooxml-chart/src/renderer/cartesian_layout.rs`의 자동 axis 계산을 `renderer.rs` 차트 출력이 소비한다. 4개 독립 출력은 가로막대·세로막대·큰 값·세로 길이 변경을 구분한다. 1/2/5 눈금 선택과 값 범위 분기를 문서 식별자 예외 없이 검토하며 일반 0-origin 차트·명시 축 설정은 정상 대조군으로 남긴다.

## 메인터너 보정과 수용 경계

공개 4종의 값축 시작값·눈금 간격만 수용 범위다. 가로막대의 plot 높이·세로 배치 등 남은 차이와 원 PR이 명시한 다른 데이터·범례·제목의 차이는 전체 해결로 확장하지 않는다. 관련 이슈 #8678은 OPEN을 유지한다.

## 공식 Print와 Native/fresh WASM 전쪽 대조

총 4종·4쪽, 최저 95.91%, 90% 미만 0쪽이다. 같은 입력·PDF의 devel 대비 2%p 초과 하락 0쪽이다. 기준 출력에 없던 쪽은 임의의 0점으로 채우지 않고 복원으로 구분한다. [Native 전쪽 비교 TSV](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/native-comparison.tsv)·[생산 SHA와 입력/PNG 해시](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/native-validation.json)를 연결한다.

| 입력 | 쪽수 | Native 최저 | fresh WASM |
| --- | ---: | ---: | --- |
| [bar_axis_floor.hwpx](../../../samples/issue8678/bar_axis_floor.hwpx) | 1 | 95.91% | 95.08% |
| [column_axis_floor.hwpx](../../../samples/issue8678/column_axis_floor.hwpx) | 1 | 99.97% | 99.99% |
| [column_axis_floor_large.hwpx](../../../samples/issue8678/column_axis_floor_large.hwpx) | 1 | 99.96% | 99.99% |
| [column_axis_step_tall.hwpx](../../../samples/issue8678/column_axis_step_tall.hwpx) | 1 | 99.93% | 99.97% |

## 기준 PDF의 출처와 제외한 자료

원 PR이 기준으로 제시한 **한컴 웹한글기안기 PDF(Cairo)는 이 검토의 검증 대상에서 제외했다.** 동일 원본의 **Windows 데스크톱 한컴 인쇄(Print) PDF**로 대조했으며, 실제 출력 제품은 한컴 2020 11.0.0.9136(MCP `engine: 2020`, HOffice110) 4종이다. 원본의 저장 제품과 Print 출력 제품을 혼동하지 않는다. [공식 Print 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)에 입력·PDF SHA-256, 엔진·제품 버전·Print method 0·작업 ID 또는 사용자 Print 확인을 보존했다.

현재 기준은 입력 34종 가운데 MCP Print 33종·사용자 확인 한컴 2024 Print 1종이다. 입력·기준 PDF는 Git에 보존하고, 이전 형식 미표기 경로에서 원본 형식·엔진을 명시한 정본 경로로 이름만 바로잡았다. [공식 Print 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)의 파일 SHA와 현재 실행·커밋이 일치한다. 이전 Cairo 기준은 Git 이력이며 현재 수용 근거로 쓰지 않는다.

## 증적과 직접 판독

[Native 직접 판독](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/native-direct-reading.json)과 [fresh WASM 직접 판독](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/wasm-direct-reading.json)에 대표 7쪽의 같은 페이지 review·overlay 확인을 기록했다. [fresh WASM 전쪽 비교](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/wasm-comparison.tsv)와 [실제 빌드·초기화 응답 해시](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/wasm-validation.json)를 함께 연결한다.

각 입력의 같은 쪽 raw·독립 PDF PNG·review·overlay 및 전체 TSV는 아래 폴더에 있다. 원 PR 주장에 대응하는 입력을 사용하며 다른 문서의 그림으로 대체하지 않는다.

- `pr8900-bar_axis_floor`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/overlay-001.png).
- `pr8900-column_axis_floor`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor/native/overlay-001.png).
- `pr8900-column_axis_floor_large`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor_large/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor_large/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor_large/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor_large/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_floor_large/native/overlay-001.png).
- `pr8900-column_axis_step_tall`: [전쪽 TSV](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_step_tall/native/candidate-silhouette.tsv) · [raw](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_step_tall/native/raw-001.png) · [PDF](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_step_tall/native/pdf-001.png) · [review](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_step_tall/native/review-001.png) · [overlay](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-column_axis_step_tall/native/overlay-001.png).

## 이 PR의 집중 회귀와 직접 판독

최소 글꼴 Native 실행에서 이슈 #8678에 대응하는 **4 PASS / 0 FAIL**을 확인했다. `30fa7739`의 공통 141건 중 이 PR의 검사를 아래에 구분한다. 선택 밖 5,964건은 실행 완료 수에 넣지 않는다. [실제 명령·검사명·로그 해시](../assets/hyunmyungjanelee_20261112/final-rebase-30fa7739/focused-validation.json)를 보존하며 이 생산 SHA를 현재 #9017 추가 검사 결과로 바꾸어 쓰지 않는다.

- `issue_8678_chart_value_axis_floor::large_column_axis_hugs_minimum`: PASS.
- `issue_8678_chart_value_axis_floor::bar_axis_starts_below_clustered_minimum`: PASS.
- `issue_8678_chart_value_axis_floor::column_axis_starts_below_clustered_minimum`: PASS.
- `issue_8678_chart_value_axis_floor::tall_column_axis_uses_finer_step`: PASS.

- `pr8900-bar_axis_floor` 1쪽: 4.1부터 5.0까지 0.1 눈금과 가로막대 끝 x 위치를 확인했다. plot 높이·세로 배치는 다르므로 차트 전체 해결로 확대하지 않고 #8678을 OPEN 유지한다.

아래 이미지는 이 PR의 같은 입력·같은 쪽이다. Native와 fresh WASM의 review·overlay를 각각 직접 열어 확인했으며 raw·PDF PNG도 함께 연결한다.

### `pr8900-bar_axis_floor` 1쪽

**native** · [raw](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/raw-001.png) · [독립 Print PNG](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/pdf-001.png)

![native review pr8900-bar_axis_floor p001](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/review-001.png)

![native overlay pr8900-bar_axis_floor p001](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/native/overlay-001.png)
**wasm** · [raw](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/wasm/raw-001.png) · [독립 Print PNG](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/wasm/pdf-001.png)

![wasm review pr8900-bar_axis_floor p001](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/wasm/review-001.png)

![wasm overlay pr8900-bar_axis_floor p001](../assets/hyunmyungjanelee_20261112/final-print-d66568ef/pr8900/pr8900-bar_axis_floor/wasm/overlay-001.png)

## 공통 최종 검증과 후속 처리

현재 제품 source `73d06ed0b1`의 최소 글꼴 Native 집중 164건·전체 13,380건과 모든 필수 게이트가 통과했다. 실제 브라우저 6사례·24회 반복 저장·API 11건도 통과했다. 이전 30fa의 시각·Studio 전체 테스트·실제 backend 증적은 동일성 감사와 원 producer를 유지해 재사용했다. 검증용 공유 캐시에 남은 기준 worktree의 root/chart library는 내용 변경 없이 현재 소스에서 실제 재컴파일했으며, 중간 실패를 제품 회귀나 최종 통과로 바꾸어 세지 않았다. 통합 PR의 최종 exact-head CI와 merge 조건을 확인한 뒤 원 PR·이슈의 정의된 해결 범위를 후속 처리한다. #8678의 부분 해결 범위는 OPEN으로 유지한다.