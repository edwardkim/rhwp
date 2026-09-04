# Task #4069 Stage 2/3 최종 보고 — 저장 프레임 경계 보존

- Issue: [#4069](https://github.com/edwardkim/rhwp/issues/4069)
- 기준: `upstream/devel` `d76d4e98b`
- 작업 브랜치: `local/task4069-redesign`
- Stage 1 중간 커밋: `7c9ce05e6`
- 기준 문서: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 한컴 정답지: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` (17쪽,
  SHA-256 `9b0390f856bb9ad43337679babf6677209b7c7ab678b6616fcc6d6d5551ff1c4`)

## 최종 결과

Stage 1의 2·3쪽 재귀 cursor를 유지하면서 두 종류의 저장 프레임 경계를 추가로 복원했다.

- 10쪽: 셀 안 같은 문단의 저장 `lineseg`가 `58620→0HU`로 되감기는 지점에서 쪽을 나눈다.
  다음 프레임의 문장은 10쪽 셀 상단에 겹치지 않고 11쪽에서 재개한다.
- 15쪽: `조달청` 제목 다음의 짧은 1×1 자식 표를 원자로 미루지 않고 현재 저장 프레임의
  남은 공간에서 시작한다. 표의 말미까지 15쪽에 이어지고 `<이해관계자 협의>`는 16쪽에서 시작한다.
- 전체: 한컴 정답지와 같은 17쪽이며, 2·3·10·11·15·16쪽의 누락·중복·겹침 계약을 모두 만족한다.

2026-08-07에 작업지시자가 2쪽의 `U+F02B1` 표식이 정상 출력된 한컴 PDF로 정답지를
교체했다. 이 교체는 위 pagination 판정을 바꾸지 않는다. 다만 기본 Canvas2D가 이 원문 PUA를
두부 글자로 그리던 별도 backend parity 결함이 드러났으며, #536 후속 stacked PR
[#4139](https://github.com/edwardkim/rhwp/pull/4139)에서 분리해 수정한다.

## 구현 계약

### 문단 내부 저장 프레임

`CellUnit`과 재귀 `NestedFlowFragment`에 `stored_frame_break_before`를 별도로 기록했다. HWP5 또는
HWP5-origin HWPX의 비합성 저장 `lineseg`가 역행하고, 직전 줄의 끝이 현재 body 높이 절반 이상에
도달한 경우만 저장 프레임 경계로 인정한다. 이 의미 경계는 일반 문단 사이 hard break의
orphan/sliver 완화 규칙으로 흡수하지 않는다.

### 저장 프레임 말미의 짧은 자식 표

문단이 정확히 하나의 1×1 자식 표를 host하고 다음 문단이 저장 프레임으로 rewind하는 경우에는,
한 페이지보다 짧은 자식 표도 canonical fragment로 푼다. 다음 프레임 경계는 엄격히 보존하되,
일반 단일 페이지 중첩 표에는 기존 원자 배치를 유지한다.

### 빈 Enter와 #2430 회귀

초기 14쪽 대조에서는 셀 안의 빈 Enter가 무시된다고 판단했지만, 작업지시자가 물리 16쪽의
셀 `(0,0)`을 다시 확인해 선두 빈 Enter 한 줄이 실제로 조판되는 것을 발견했다. IR에도 빈
`p[0]`의 저장 줄(`vpos=0`, `line_height=900HU`)과 다음 `p[1]`의 `vpos=1800HU` 전진이 남아
있었다. 기존 구현은 비인라인 1×1 RowBreak 표의 빈 문단을 일괄 0높이로 접어 첫 문장을 표
상단에서 `0.94px` 아래에 그렸지만, 한컴 PDF는 약 `27px`의 선두 빈 줄을 둔다.

수정은 두 의미를 분리한다. 빈 문단 뒤 저장 줄이 최소 한 줄높이만큼 순방향으로 전진하면 실제
줄박스 높이는 보존한다. 반면 빈 문단의 `vpos` rewind는 그 자체만으로 저장 프레임·페이지
경계로 승격하지 않는다. 그 결과 물리 16쪽의 첫 문장 간격은 `24.9px`로 회복하면서도 문서는
한컴 정본과 같은 39쪽을 유지한다. `issue_2430_page16_keeps_leading_blank_paragraph_in_cell`이
빈 줄 높이를, `issue_2430_cell_rewrap_threshold_no_oversplit`이 39쪽 계약을 각각 고정한다.

## 검증

### 자동 테스트

- `cargo test --profile release-test --tests`: exit 0
  - 라이브러리 3,293개: 3,285 passed, 8 ignored, 0 failed
  - 모든 통합 test binary 통과
- 핵심 focused 회귀 34개 통과
  - #4069 4개: 17쪽, 2·3쪽 cursor, 10·11쪽 frame, 15·16쪽 child table
  - #2430 2개: 39쪽, 물리 16쪽 셀 선두 빈 Enter 높이
  - #1891, #2279, #3637, `issue_rowbreak_chart_overlap` 포함
- `overflow_cell_baseline` 통과: 장식 간격용 짧은 빈 문단과 control host 문단은
  새 full-line 보존 조건에서 제외해 `hwpx_sample2.hwp`의 기존 overflow 3줄 계약 유지
- Native Skia: 라이브러리 58 passed, #2225 2 passed, direct PDF 4 passed
- `cargo fmt --check`, `git diff --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --doc`: 4 passed, 2 ignored

### WASM

프로젝트 표준 Docker Compose 절차로 `wasm-pack 0.15.0` 빌드를 새로 수행했다.

- 최종 `pkg/rhwp_bg.wasm` SHA-256:
  `17e14d48222321195f8d42f6f1e998a883a720472fc67aab7d46d41c1b423549`
- 생성된 `rhwp.js`와 `rhwp.d.ts`: 기존 바인딩과 차이 없음
- Node 직접 로드: #4069 17쪽, #2430 39쪽
- WASM HTML 물리 16쪽: 빈 TextLine `top=200.33px`, 첫 문장 `top=224.33px`,
  표 상단부터 첫 문장까지 `24.94px`
- WASM render: 15쪽에 조달청 자식 표 시작·말미 존재, 다음 프레임 부재;
  16쪽에 `<이해관계자 협의>` 존재

일반 `rhwp-studio npm run dev`는 루트 `pkg`를 alias한다. 빌드 산출물은 메인 작업공간의
`pkg/rhwp_bg.wasm`에 적용했으며, 실행 중인 WASM 인스턴스는 dev 서버 재시작과 브라우저 강력
새로고침 후 교체된다. `dev:subsecond`는 별도 WASM 경로이므로 이 적용 대상이 아니다.

### 시각 검증

`output/4069/stage3-final-validated/`에서 한컴 2020 PDF 17쪽 전부를 비교했다.

- SVG·render tree·PDF raster·compare·overlay·review: 각각 17쪽
- 누락 페이지: 0
- 자동 구조 후보 `flagged_page_count`: 0
- 수행자 직접 검토: 2·3·10·11·15·16쪽 흐름 정합

폰트·안티앨리어싱 차이로 pixel/ink 일치율 자체는 완료 판정으로 쓰지 않았다. 정확 쪽수,
render-tree 텍스트 계약, 자동 구조 후보, 한컴 PDF review를 함께 판정 근거로 사용했다.

#2430의 추가 빈 Enter 판정 근거는 `output/4069/issue2430-p16-fixed/`에 일반·디버그
PNG/SVG와 render tree, 수정 전·한컴 PDF 바로가기로 남겼다.

PR 준비 시 최신 `upstream/devel` `d634e608b`를 병합한 head `eb699faa2`에서 위 전체
Cargo·Native Skia·WASM 검증을 다시 실행했다. 메인 `pkg`의 새 WASM을 직접 로드해도
#4069 17쪽, #2430 39쪽과 물리 16쪽 `24.94px` 간격이 유지됐다.

## 단계 판정

로컬 구현·회귀·WASM·시각 검증은 완료했다. 메인 작업공간의 기존 Claude Code/user WIP는
되돌리거나 덮어쓰지 않았다. 원격 push, PR 생성, GitHub comment와 이슈 close는 별도 승인을
받기 전까지 수행하지 않는다.
