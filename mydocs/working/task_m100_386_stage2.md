# Task #386 단계 2: 단위 테스트 추가 (Red) — 완료보고서

> **이슈**: [#386](https://github.com/edwardkim/rhwp/issues/386)
> **브랜치**: `local/task386`
> **작성일**: 2026-04-27

---

## 목표

`compute_body_wide_top_reserve_for_para`의 `VertRelTo::Paper` 좌표계 버그를 노출하는 단위 테스트 추가. 수정 전 Red 상태 확보.

## 변경 파일

`src/renderer/typeset.rs` — `mod tests` 끝부분에 Task #386 테스트 그룹 추가.

추가된 테스트:

| 테스트 | 의도 | 단계 2 상태 |
|--------|------|--------------|
| `t386_body_wide_reserve_paper_relative_returns_body_relative` | Paper-rel 표가 body-rel reserve를 반환해야 함 (exam_eng pi=0 ctrl[4] 재현) | **FAIL (Red)** |
| `t386_body_wide_reserve_paper_relative_inside_header_skipped` | 머리말 영역 전체 도형은 reserve=0 (가드 보존) | PASS |
| `t386_body_wide_reserve_para_relative_unchanged` | Para-rel 도형은 변환 없이 그대로 (회귀 방지) | PASS |

테스트 헬퍼:
- `a3_page_def_exam_eng()`: A3 297×420mm + 56.5mm 상단 여백
- `two_column_def()`: 2단 11.0mm 간격
- `make_para_with_top_bottom_table(...)`: TopAndBottom 비-TAC 표 단일 문단 생성

## 핵심 테스트 케이스 수치

```
입력 (exam_eng.hwp pi=0 ctrl[4] 재현):
  vert_rel_to    = Paper
  vertical_offset = 10885 HU = 145.13 px (page-abs)
  width          = 66616 HU
  height         = 11058 HU = 147.44 px
  margin.bottom  =  1132 HU =  15.09 px

A3 layout body_top = 16013 HU = 213.51 px

기대값 (수정 후):
  bottom_abs = 145.13 + 147.44 + 15.09 = 307.67 px
  reserve    = max(0, 307.67 - 213.51) = 94.13 px (body-rel)

현재값 (수정 전):
  reserve    = 307.67 px (page-abs 그대로)
```

## 실행 결과

```
$ cargo test --release --lib t386_body_wide_reserve

running 3 tests
test renderer::typeset::tests::t386_body_wide_reserve_paper_relative_inside_header_skipped ... ok
test renderer::typeset::tests::t386_body_wide_reserve_para_relative_unchanged ... ok
test renderer::typeset::tests::t386_body_wide_reserve_paper_relative_returns_body_relative ... FAILED

failure:
  expected≈94.13 px, got 307.67 px (page-abs 좌표 그대로 반환되면 307.67)

test result: FAILED. 2 passed; 1 failed; 0 ignored
```

## 회귀 확인

```
$ cargo test --release --lib

test result: FAILED. 1016 passed; 1 failed; 1 ignored
```

- 신규 테스트 1건 의도된 실패 (Red)
- 기존 1015 테스트 + 신규 2 테스트 PASS
- 기존 테스트 회귀 0건 ✅

## 산출물

- 본 보고서 (`mydocs/working/task_m100_386_stage2.md`)
- 소스 변경: `src/renderer/typeset.rs` — 단위 테스트 3건 + 헬퍼 함수 추가
- 다음 단계: 단계 3 — `compute_body_wide_top_reserve_for_para` 수정 (Green)
