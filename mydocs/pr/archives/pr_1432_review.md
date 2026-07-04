# PR #1432 검토 — 차트 샘플 코퍼스 27종 추가 (P-1, refs #1431)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1432
- 작성자: `johndoekim` — 표·그림·글상자 hit-test 영역 활발한 컨트리뷰터(머지 다수)
- 상태: open / 라벨 없음
- base: `devel` ✓ (head: `feature/chart-samples`, 작업 커밋 `616aa396` + devel 동기화 merge)
- mergeStateStatus: **CLEAN** (BEHIND 아님)
- 연결 이슈: `refs #1431` (차트 OLE/OOXML 지원 트래킹, OPEN — closes 아님, 선행 P-1)
- 변경: 82파일 +8/-0 (README 8줄 + 바이너리 81)

## 2. 변경 요약

한컴 2022 편집기 제작 차트 27종(가로/세로막대·라인·원형·분산형·주식형) 검증 코퍼스.

- `samples/chart/{종류}/{이름}.hwp` + `.hwpx` — **27쌍**(hwp 27 + hwpx 27 확인).
- `pdf/chart/{종류}/{이름}-2022.pdf` — 한컴 2022 참고 출력 27개.
- `samples/chart/README.md` — 출처·ground-truth 메모.
- 모든 샘플이 OOXMLChartContents(DrawingML) + 레거시 Contents 보유 → 양 경로 검증.
- **데이터 전용 PR — Rust 코드 변경 0.**

## 3. 검증

- **무관 변경 0**: 전부 `samples/chart/` + `pdf/chart/`. 비-차트 파일 섞임 없음.
- **PDF 크기/LFS**: pdf/chart 합계 2.5MB, 최대 110KB — 전부 50MB 미만 → `pdf/`(일반 git)
  배치 정확. `.gitattributes` LFS 패턴은 `pdf-large/**` 한정이라 **LFS 쿼터와 무관**.
- **쌍 완전성**: hwp 27 + hwpx 27 = 27쌍 완전.
- **회귀 없음(코드 무변경) 주장 검증**: samples/ 를 동적 탐색하는 테스트는
  `lineseg_compare_tests.rs::test_lineseg_compare_hancom_saved` 1건뿐인데, `read_dir("samples/")`
  **직속(non-recursive)** + `re-*-hancom.hwp` 패턴만 매칭 → `samples/chart/{종류}/` 하위 차트
  샘플은 위치·패턴 모두 불일치. 실제 실행 시 **1 passed**(끌려들지 않음 확인).
- **샘플 동작**: 종류별 대표 5종(세로막대·원형·라인·분산형·주식) `export-svg -p 0` 전부
  **OK**(파싱·렌더 패닉/에러 없음).
- `cargo fmt --check`: OK. GitHub CI: Build & Test / Analyze(rust·js·python) / CodeQL 전부 pass.

## 4. 평가

- 데이터 전용·무관 변경 0·폴더 규칙 정합(samples/chart, pdf/chart). PDF 50MB 미만으로 LFS
  쿼터 무관 — 현 LFS 초과 상황에서 안전.
- 27쌍 완전 + OOXML/레거시 양 경로 보유 → #1431(차트 지원) 회귀 가드로 가치 큼.
- "회귀 없음" 주장이 실제로 정확(동적 탐색 테스트 미해당, fmt/CI 통과, 표본 export OK).
- README ground-truth 메모 적절. PDF 는 참고 출력으로 명시(정답지 주장 아님).

## 5. 판단

**merge 권고**. 순수 검증 데이터, 규칙 준수, 회귀 0(검증 완료), LFS 무관. `refs #1431`
트래킹 이슈는 유지(P-1 선행). 세부는 `pr_1432_report.md`.
