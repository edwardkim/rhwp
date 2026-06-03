# Stage 2 완료보고서 — Task #1219

**제목**: Stage 1b(선두 미주 마커 측정 이중계상 제거) + 전체 회귀 검증
**이슈**: [#1219](https://github.com/edwardkim/rhwp/issues/1219)
**브랜치**: `local/task1219`
**작성일**: 2026-06-01

---

## Stage 1b — 선두 미주 마커 측정 이중계상 제거 (승인된 스코프 확장)

`src/renderer/layout/paragraph_layout.rs` — `footnote_positions` 측정 루프(~1887).

**변경**: `start_line == 0` 이고 control 이 `Control::Endnote` 인 경우(= `endnote_marker_x_advance`
가 풀사이즈 선두 마커로 렌더하고 그 폭을 `inline_offset` 에 이미 반영한 미주) 측정 루프에서
`continue` 로 제외.

```rust
let is_leading_endnote_marker = start_line == 0
    && matches!(para.and_then(|p| p.controls.get(ctrl_idx)), Some(Control::Endnote(_)));
if is_leading_endnote_marker { continue; }
```

**근거**: 렌더 경로는 이 미주의 인라인 위첨자를 그리지 않는다(SVG 검증: 문26 "공" x=78 =
선두 마커 끝, 위첨자 부재). 측정만 fn_text 위첨자 폭(~22px)을 더해 이중 계상 → 거짓 오버플로우.
측정을 렌더 동작에 일치시킴.

## 누적 결과 — 본문 한글 advance (PDF 목표 12px)

| 줄 | 원본 | Stage 1 (TAC) | Stage 1b (+미주) |
|----|------|--------------|-----------------|
| 문26 공차가… | 8.96px | 11.08px | **11.93px** ✅ |
| 문27 함수… | 10.27px | 10.27px | **11.85px** ✅ |
| 문24 매개변수… | 8.77px | 10.25px | 11.30px |
| 문30 길이가… | 9.15px | 10.52px | 11.28px |

- 문26(사용자 지적 줄)·문27: ≈12px 정합. 시각 겹침 완전 해소(`output/poc/eq26_1b/cmp_q26_1b.png`,
  rhwp ↔ PDF 한글 2022 — 간격 거의 동일).
- 문24/30: 8.77/9.15 → 11.3px 로 대폭 개선. 잔여 ~0.7px 미세 압축은 인라인 수식 폭 측정의
  별개 미세 오차로 추정(겹침 없음) — 본 이슈(거짓 오버플로우·겹침) 범위 밖, 추적 시 별도 이슈.
- 미주 마커 "문26)" 는 선두 1회 렌더 유지, 레이아웃 이동 없음.

## 회귀 검증

```
cargo test --release → 전체 1896 passed / 0 failed
```

- 골든 SVG 스냅샷(`tests/svg_snapshot.rs`) 8건 전부 통과:
  table_text / issue_157 / issue_267(KTX 목차) / issue_677(복학원서) /
  form_002 / issue_147(aift) / **issue_617(exam_kor p5)** / 결정성.
  → 인라인 TAC·각주·목차·우측탭 등 렌더 정합 회귀 없음 확인.
- `tab_cross_run`(Task #290 인라인 LEFT 탭) 통과.

**경계 케이스 — 마지막 줄 끝 수식**: `line_tac_offsets` 가 `composed_line_char_end` 의
마지막-줄 분기(`+ has_line_break`)로 해당 케이스를 렌더와 동일하게 포함 → 회귀 위험 없음.
미주 수정은 `start_line==0` 선두 미주로 한정 → 일반 각주/인라인 미주 영향 없음.

## 다음 단계 (Stage 3)

- 회귀 테스트 추가(문26 수식 줄 한글 advance 비압축 검증).
- 변경 파일 한정 rustfmt.
- 최종 보고서.
