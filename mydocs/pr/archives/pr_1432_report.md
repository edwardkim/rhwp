# PR #1432 최종 보고서 — 차트 샘플 코퍼스 27종 (P-1, johndoekim)

## 1. 결정

**merge 수용** — 작업 커밋 `616aa396` 을 `local/devel` 에 cherry-pick → `devel` push.

## 2. 변경 본질

한컴 2022 차트 27종(가로/세로막대·라인·원형·분산형·주식형) 검증 코퍼스 추가. 데이터
전용(Rust 코드 0). `samples/chart/{종류}/{이름}.hwp`+`.hwpx` 27쌍 + `pdf/chart/` 참고 출력
27 + README. OOXMLChartContents + 레거시 Contents 양 경로 보유. refs #1431 선행 P-1.

82파일 +8/-0 (README 8줄 + 바이너리 81).

## 3. 검증

- 무관 변경 0 (전부 samples/chart·pdf/chart).
- PDF 합계 2.5MB / 최대 110KB — 50MB 미만, pdf/(일반 git) 배치 정확, **LFS 쿼터 무관**.
- 쌍 완전: hwp 27 + hwpx 27.
- 회귀 0: samples/ 동적 탐색 테스트(`test_lineseg_compare_hancom_saved`)는 `re-*-hancom.hwp`
  패턴 + samples/ 직속만 → 차트 샘플 미해당, 실행 시 1 passed.
- 표본 export-svg 5종(전 종류) OK. `cargo fmt --check` OK. GitHub CI 전부 pass.

## 4. merge 방식 — cherry-pick

mergeStateStatus=CLEAN 이었으나, 검증된 작업 커밋 1개만 `local/devel` 에 cherry-pick
(author johndoekim 보존)하여 push. PR 은 "devel 에 포함됨" 으로 close.

## 5. 후속

- `refs #1431`(차트 OLE/OOXML 트래킹) 유지 — 본 PR 은 선행 P-1, 이슈 닫지 않음.
- 차트 렌더 커버리지 현재 13/27 — 미지원 14종은 #1431 에서 추적.
- merge 후 리뷰/보고서 archives 이동, 오늘할일 반영.
