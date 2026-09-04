# Task M100 #1380 3단계 완료 보고서 — diff_linesegs baseline 게이트 동승 (xfail 0)

- 구현계획서: `mydocs/plans/task_m100_1380_impl.md` 3단계
- 브랜치: `local/task1380`

## 1. 구현 내용 (`src/serializer/hwpx/roundtrip.rs` 단일 파일)

| 변경 | 내용 |
|------|------|
| `IrDifference::ParagraphLinesegs` 신설 | `{ section, paragraph, path, detail }` — `path` 표기는 `ParagraphCharShapes`(#1378)와 동일, `detail` 은 `LinesegDiffKind` 표시 문자열. `Display` arm 동반 |
| `LinesegDiffKind` 에 `Display` 구현 | `count: expected=.. actual=..` / `[i].field: expected=.. actual=..` 형식 |
| `diff_documents` 말미 확장 | `diff_linesegs(a, b)` 결과를 `ParagraphLinesegs` 로 변환·추가 — 게이트 동승의 단일 지점 |
| 주석 갱신 | 모듈 헤더에 #1380 확장 항목 추가, `LinesegDiff`/`diff_linesegs` 의 "게이트 비동승, 측정 전용" 문구를 동승 반영으로 교체 |

**동승 경로 (코드 수정 없이 자동 반영된 소비자 3곳):**

- `roundtrip_ir_diff` (단일 파일 검사) — `diff_documents` 경유
- `tests/hwpx_roundtrip_baseline.rs` (전수 게이트 + 2-round 안정성) — `diff_documents` 경유
- 배치 진단 `hwpx_roundtrip_batch.rs` 의 IR_DIFF 판정 — `diff_documents` 경유.
  `--lineseg-report` 의 필드 단위 TSV 측정(`diff_linesegs` 직접 호출)은 상세 보고용으로 유지

## 2. 단위 테스트 (구현계획서 3.3 대응)

| 테스트 | 고정하는 동작 |
|--------|--------------|
| `task1380_lineseg_in_gate` (구 `diff_linesegs_not_in_gate` 반전) | lineseg 값 차이가 `diff_documents` 에서 `ParagraphLinesegs` 로 검출 — detail 문자열 포맷 포함 |
| `task1380_gate_detects_synthetic_lineseg_asymmetry` (신설) | #1380 결함 본체였던 원본 무 → RT 유 합성 비대칭을 개수 불일치로 검출 |
| `task1380_two_round_stable_with_empty_linesegs` (신설) | linesegarray 부재 문단 38건을 실재 보유한 H1 샘플(business_overview)의 round1·round2 게이트 0 — 빈 `line_segs` 가 어느 라운드에서도 합성·소실되지 않음 |

기존 `diff_linesegs_*` 측정 단위 테스트 4건(값/개수/셀 재귀)은 불변 유지.

> 2-round 테스트 픽스처 비고: 수제 `Document`(빈 doc_info) 기반 2-round 는 serializer 의
> ID 등록 검사(미등록 charPrIDRef)와 충돌하는 별개 한계가 있어, 실샘플 기반으로 작성.
> linesegarray 부재 문단 존재를 테스트 전제(assert)로 명시해 픽스처 드리프트를 방어.

## 3. 검증

### 3.1 게이트 동승 결과 — 전수 xfail 0

```
cargo test --test hwpx_roundtrip_baseline → 4/4 passed (35.6s)
  baseline_all_samples_roundtrip / baseline_large_samples_roundtrip
  xfail_entries_still_fail / grade_lists_are_consistent
```

XFAIL 목록 변경 없음 (잔존 5건은 전부 lineseg 무관 기지: #1382 ×1, #1384 ×4) —
**구현계획서의 "현재 전수 lineseg diff 0 → xfail 0 동승" 전제가 게이트에서 실증됨.**

### 3.2 배치 전수 (output/poc/task1380_s3)

```
총 54 → PASS 48 / IR_DIFF 1(#1382) / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1  (2단계와 동일)
lineseg_diff.tsv → round1=0 round2=0 파일=0  (드리프트 0)
```

IR_DIFF 판정이 lineseg 차이를 포함하게 된 후에도 분류 불변 — 동승이 기존 분류를
흔들지 않음을 확인 (143E… 의 diff=1 은 기지 #1382 char_shapes 차이 그대로).

### 3.3 CI급

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1716 passed (2단계 1714 + 신규 2) |
| `cargo test --tests` | 2226 passed / 0 failed (baseline·sample16 페이지 수 게이트 포함) |
| `cargo fmt --check` | 통과 |
| `cargo clippy --lib --tests` | 경고 0 |

## 4. 산출물

- 소스: `src/serializer/hwpx/roundtrip.rs` (variant·Display·동승 + 테스트 3)
- 측정: `output/poc/task1380_s3/` (inventory.tsv, lineseg_diff.tsv — git 미포함)

## 5. 다음 단계

4단계 (최종): 전수 검증 종합 + 이슈 4샘플 페이지 수 비교 + 대표 SVG 비교 +
한컴 편집기 판정 요청(작업지시자) + `manual/hwpx_roundtrip_baseline.md` 갱신
(게이트 비교 항목에 lineseg 추가, known limitations 의 #1380 행 해소) + 최종 보고서.
