# Task #552 Stage 1 완료 보고서

**제목**: TDD RED 테스트 + 광범위 사전 평가
**브랜치**: `local/task552`
**이슈**: https://github.com/edwardkim/rhwp/issues/552

---

## 1. TDD 통합 테스트 추가 (RED 확인)

`integration_tests.rs` `test_552_passage_box_top_gap_p2_4_6` 추가.

페이지 2 우측 단 [4~6] header text 와 박스 top horizontal line 간 gap 검증:
- [4~6] header "[" 위치 추출 (우측 단 x ≥ 575, y in [215, 230])
- 박스 top horizontal line 추출 (header_y < y < header_y+30, x ≥ 575)
- gap = box_top_y - (header_y + 2.20 ascent) ≥ 6.0 px (PDF 8.73 px ±2 px)

```
test test_552_passage_box_top_gap_p2_4_6 ... FAILED
[4~6] 박스 top y=224.43 가 header bottom y=224.43 와 충분한 gap 을 가져야 함.
gap=0.00 px (PDF 기대 8.73 px ±2 px). 버그(수정 전): gap=0.0
(Task #479 가 본문 paragraph 마지막 줄 trailing ls 제외 → border-start
paragraph 가 9.54 px 위로 이동).
```

`#[ignore]` attribute. 1119 단위 테스트 baseline 통과.

## 2. 광범위 사전 평가

`examples/scan_border_starts.rs` 도구로 6 샘플의 paragraph border 시작 패턴 분석.

### 2.1 영향 케이스 분포

| 샘플 | total | no→border (회귀 영향) | in_border | border→no | no→no |
|------|-------|----------------------|-----------|-----------|-------|
| 21_언어_기출 | 325 | **10** | 59 | 10 | 245 |
| exam_kor | 749 | **14** | 225 | 14 | 493 |
| exam_math | 275 | **8** | 4 | 8 | 253 |
| exam_eng | 318 | **16** | 2 | 16 | 283 |
| exam_science | 130 | 0 | 0 | 0 | 129 |
| synam-001 | 250 | 0 | 0 | 0 | 249 |
| **합계** | **2047** | **48** | 290 | 48 | 1652 |

### 2.2 fix 영향 범위

- **48 cases** (no-visible → visible border 전환): 박스 top 이 9.54 px 하향 이동 (PDF 정합 회복)
- **290 cases** (in_border): 변경 없음 (border 내부, Task #479 그대로)
- **48 cases** (border → no): 변경 없음 (border 끝, Task #479 그대로)
- **1652 cases** (no → no): 변경 없음 (Task #479 의 본 효과 보존)

### 2.3 Task #479 본 효과 (페이지 12 200 px drift) 보존 검증 가능

Task #479 가 해결한 페이지 12 200px drift 는 다중 paragraph 누적 결과. 본 fix
는 transition 시점에서만 trailing ls 복원 → no→no 케이스 (1652 cases) 는 영향
없음 → drift 누적 본질 보존.

### 2.4 fix 후보 A 검증 가능

```rust
} else if is_full_paragraph_end && cell_ctx.is_none()
    && !next_paragraph_starts_visible_border() {
    y += line_height;     // trailing ls 제외 (#479)
} else if is_full_paragraph_end && cell_ctx.is_none() {
    let line_spacing_px = ...;
    y += line_height + line_spacing_px;  // border-start 직전: ls 보존 (#552)
} else {
    ...
}
```

`next_paragraph_starts_visible_border()` 산출 위해 caller (`layout_partial_paragraph`)
에서 다음 paragraph 의 border_fill_id 정보 전달 필요.

## 3. 회귀 위험 분석

### 3.1 영향 케이스 별 평가

| 케이스 | 영향 |
|--------|------|
| no→visible-border (48 cases) | **변경됨** — trailing ls 보존, 박스 top 9.54 px 하향 |
| 그 외 1999 cases | 변경 없음 |

### 3.2 Task #544 / #547 / #548 무회귀 검증 후보

- Task #544 [7~9] 페이지 4 박스 (`test_544_passage_box_coords_match_pdf_p4`):
  본 fix 영향 케이스 (pi=80 본문 → pi=81 박스 시작 등). 박스 top y 가 9.54 px
  하향 → PDF 기대값과 비교 필요.
- Task #547 본문 inset, Task #548 [푸코]: 박스 외부 변경 — 영향 없음 예상.

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | RED 테스트 1건 (+97 LOC) |
| `examples/scan_border_starts.rs` | 광범위 평가 도구 (신규) |
| `mydocs/working/task_m100_552_stage1.md` | 본 보고서 |

## 5. 다음 단계 (Stage 2)

1. `paragraph_layout.rs` `is_full_paragraph_end` 분기 보강 (후보 A)
2. caller signature 변경 — 다음 paragraph border_fill_id 전달
3. RED → GREEN 확인
4. Task #544 회귀 사전 검증 (`test_544_passage_box_coords_match_pdf_p4`)
5. 1119 단위 테스트 무회귀
6. Stage 2 보고서 + 커밋

## 6. 승인 요청

Stage 1 완료. 후보 A (next_paragraph_starts_visible_border 조건 추가) 진행 OK?

승인 후 Stage 2 (fix 적용) 진행합니다.
