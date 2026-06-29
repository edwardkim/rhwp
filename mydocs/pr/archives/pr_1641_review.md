# PR #1641 처리 보고서 — 빈-host 음수 offset float 표 문서순서 역전 회귀 수정 (#1639)

- PR: https://github.com/edwardkim/rhwp/pull/1641
- 제목: `Task #1639: 빈-host 문단 음수 offset float 표가 문서/배열 순서를 역전하던 회귀 수정`
- 작성자: kkyu8925 (collaborator, #1535/#1549 float 레이아웃 시리즈 연속)
- 연결: Closes #1639 (PR #1518/#1510 이후 빈-host 잔존 케이스)
- base ← head: `devel` ← `kkyu8925:fix/1639-empty-host-negative-offset-float-order`
- 처리일: 2026-06-29

## 1. 처리 결정 — 시각 판정 통과 → admin merge

**admin merge (시각 판정 통과).** 선례 #1568(visible-host) 과 동일하게, 본 PR 은 표
배치·페이지 순서를 바꾸는 렌더링 변경이므로 작업지시자 시각 판정을 거쳤다. 코드/회귀
검증 모두 green + SVG·PNG 시각 대조로 역전→복원 입증 + **작업지시자 시각 판정 통과
(2026-06-29)**. 컨트리뷰터(macOS·한컴 미접근)의 메인테이너 재검증 요청도 충족.

### 시각 판정 결과 (PNG Skia raster 대조)

| | page1 표 순서(위→아래) | 판정 |
|---|---|---|
| 수정 전(devel) | **B → A → C** | ❌ 음수 표 B(ci=3, voffset −4411)가 양수 표 A(ci=2, +200) 위로 점프 — 역전 |
| 수정 후(PR) | **A → B → C** | ✅ 문서 순서(ci=2→3→4) 복원 |

- 자료: `output/poc/task1639/{before,after}/issue1639_*_001.png`
- 4경로 일치: dump-pages·render tree·SVG y 좌표·PNG(Skia) 모두 `[3,2,4]`→`[2,3,4]`.

## 2. 변경 범위 (5 files +231/-1)

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | 빈-host float 표 정렬 게이트 1곳에 `has_negative_para_float` AND 추가. 음수 `vertical_offset`(signed) 표가 하나라도 있으면 정렬 OFF → 배열(문서) 순서 보존 |
| `tests/issue_1639.rs` | 신규 2 — 표 배치 순서 `[2,3,4]` 보존 + render tree y 좌표 문서 순서 |
| `samples/issue1639_empty_host_negative_offset_float.hwpx` | fixture (issue1549 clone-and-narrow, 빈 host + ci=2/+200·ci=3/-4411·ci=4/+800) |
| `mydocs/plans·report` | 계획·보고서 |

## 3. 근본 원인 / 수정

`should_sort_para_float_tables = !para_has_non_whitespace_text(para)` (빈 host 정렬,
#986/#1088) + 정렬 키 `vertical_offset as i32`(음수 키) + `sort_by_key` 오름차순 →
음수(-4411)가 양수(+200) 앞으로 정렬되어 `ctrl_order` 역전. 파싱은 정상(offset 비트 보존),
정렬 휴리스틱의 "voffset 오름차순 = 문서 순서" 가정이 음수에서 깨짐.

수정: 게이트에 `&& !has_negative_para_float` 추가. 양수 전용 빈 host 정렬(#986/#1088)·
visible-host 제외(#1510)·정렬 키·배치 downstream(`is_first/last_placed`, `FloatLaneSet`)
전부 불변. 최소 침습.

## 4. 로컬 검증 (메인테이너 환경, Linux WSL2)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 | 0건 (CLEAN) |
| 신규 `issue_1639` | 2 passed |
| 회귀 `issue_986/1510/1535/1549` + `hwpx_roundtrip_baseline` | 전부 ok |
| 전체 `cargo test --tests` | **FAILED 0건** |
| fmt --check / clippy(typeset.rs) | clean / 무경고 |
| **회귀 재현·복원 (dump-pages)** | devel: `[3,2,4]`(역전) → PR: `[2,3,4]`(문서순서 복원) — 음수 ci=3 표가 양수 ci=2 앞으로 점프하던 것 해소 |

## 5. 시각 판정 자료 (작업지시자 확인 요청)

- 픽스처: `samples/issue1639_empty_host_negative_offset_float.hwpx`
- 수정 후 SVG: `output/poc/task1639/after/issue1639_*_001.svg` (2페이지, page1에 표)
- 판정 포인트: page1 의 표 3개가 **문서 순서대로 위→아래**(작은 표 → 중간 표 → 1x1)
  배치되는지. 음수 offset 표가 선행 형제 위로 점프하지 않아야 한다(수정 전 역전 양상).
- 권위: 페이지/배치 정합 최종 판정은 작업지시자 한컴 환경. 로컬은 dump-pages·render
  tree·SVG y 좌표 3경로로 `[3,2,4]`→`[2,3,4]` 복원만 확인.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1641_review.md`
