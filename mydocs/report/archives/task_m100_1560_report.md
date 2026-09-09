# Task #1560 최종 결과보고서 — 한글 페이지 충실도 오라클 정식화

- 이슈: #1560 (M100)
- 브랜치: `local/task1560` (from devel = #1552+#1554+#1557)
- 일자: 2026-06-26

## 1. 목표 및 결과
IR 게이트가 못 잡는 **한글 전용 페이지 붕괴**(#1557 secCnt류, 잔여 2→1)를 재현·게이트
가능하게 검출하는 정식 도구 `tools/verify_hangul_pages.py` 를 신설. 흩어진 임시
스크립트(`output/poc/fidelity*/t3_*.py`)를 일원화.

## 2. 도구 개요
- 입력: `--batch <원본> <rt>` | `--inventory <tsv> --orig-root --rt-root`(roundtrip 호환).
- 검사: 원본↔rt 한글 `PageCount` 비교 → `OK`/`COLLAPSE`/`EXPAND`/`ERR`.
- 옵션: `--status` 필터, `--sample N --seed S`(재현 표본), `--pdf`(PyMuPDF 교차검증).
- 출력: TSV + 붕괴율 요약 + **COLLAPSE 시 종료 코드 1**(게이트). 헤더에 **git HEAD** 기록.
- 견고성: 파일별 예외 격리, pyhwpx/PyMuPDF 미설치 명확 안내.

## 3. 검증
- 알려진 케이스(배치): 36382669 **OK**(8→8, secCnt 수정), 36384160 **COLLAPSE**(29→3),
  36387103 **COLLAPSE**(2→1) → 종료 코드 1.
- 인벤토리+표본 재현: `--sample 45 --seed 42` 가 v3 임시 스크립트 결과(**1/45=2% 붕괴**)를 정확 재현.
- git HEAD(b086bd5a) 기록 확인.

## 4. 가치
페이지 붕괴 같은 **IR-blind 최악 클래스를 재현 가능·게이트 가능**하게 만들었다. 향후
secCnt류 회귀를 한컴 보유 환경에서 즉시 감지. `hwpx-roundtrip`/`hwp5-roundtrip` 산출
inventory 와 직접 연동.

## 5. 한계 / 후속
- 한글 의존(Windows+한컴) — Linux CI 게이트 불가, 로컬 오라클.
- 페이지수 비교 한정 — **시각 픽셀 diff(T4)**·**고정 실문서 회귀 말뭉치**·**pic 시각 triage**는
  별도 이슈(측정도구 고도화 2·3순위).

## 6. 변경 파일
- 신규: `tools/verify_hangul_pages.py`, `mydocs/manual/verification/hangul_page_oracle.md`
- 계획/보고: `mydocs/plans/task_m100_1560{,_impl}.md`, `mydocs/working/task_m100_1560_stage{1..3}.md`
- rhwp 소스 무변경(순수 도구/문서).
