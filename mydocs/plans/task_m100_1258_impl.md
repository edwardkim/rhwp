# 구현계획서 — Task #1258: typeset 미주 base-flow trailing IR 명시 (A 정규화)

- **이슈**: edwardkim/rhwp#1258
- **브랜치**: `feature/issue-1258-trailing-base-flow-normalize` (base `stream/devel`)
- **수행계획서**: [`task_m100_1258.md`](task_m100_1258.md)
- **원칙**: 동작 무변경 리팩터. **빅뱅 금지** — 재현 가드 먼저, 한 분기씩 제거하며 green 유지.

---

## 핵심 메커니즘 (확정)

```
typeset.rs:2226  prev.last_seg.line_spacing = between_notes   ← trailing 을 IR 에 bake (단일줄·다줄 공통)
typeset.rs:2220  vpos_offset += pagination_gap(=between_notes-1984)  ← 초과분만 다음 미주 vpos 에 가산
                 └ 가정: base-flow 1984HU 는 prev 의 자연 line_seg 흐름으로 vpos 에 이미 있다.

render height_cursor.rs:163  vpos_continuous && prev_has_text → trailing_ls_hu = 0 (이미 포함 간주)
                 └ 단일줄 prev: 마지막=유일 seg 라 trailing 이 sequential y 에 반영 → 일치
                 └ 다줄 prev: render 가 마지막 줄 trailing 을 y 누적에서 누락 → gap≈0 (#1246)
render S8 계열  :448-466(#1256/#1261 단일줄), :477-484(#1246 다줄)  ← 이 불일치를 하류 보정
```

목표: **base-flow 1984HU 가정을 단일줄·다줄 모두에서 참이 되게** 만들어 S8 계열을 제거.

---

## 구현안 (Stage 2 에서 spike 후 택1)

| 안 | 내용 | 장점 | 위험 |
|----|------|------|------|
| **B (권장)** | render `:163` 게이트가 **다줄 prev 의 마지막 줄 trailing 도 단일줄과 동일하게** y 누적에 포함하도록 정합 (render 측 정규화) | vpos/pagination 절대값 불변 → 페이지 분할 영향 없음 | render layout 의 다줄 마지막 줄 trailing 누락 지점 특정 필요 |
| A | typeset `vpos_offset` 가 **full gap** 을 encode (base-flow 도 vpos 에 가산) | render 추측 완전 제거 | vpos 절대값 변동 → pagination 페이지 분할 이동 위험(C 영역) |
| C | 하이브리드(typeset 은 다줄 마지막 seg 만 vpos 노출) | 국소 변경 | 두 곳 동시 수정, 검증 복잡 |

> 기본 방향 **B**: pagination/vpos 절대값을 건드리지 않아 위험이 가장 낮다. Stage 2 spike 로
> "다줄 prev 마지막 줄 trailing 이 render y 누적에서 빠지는 정확한 지점"을 특정한 뒤 확정.

---

## 단계 (5단계)

### Stage 1 — 재현 가드 확립 (코드 무수정 또는 테스트만 추가)

현 동작을 **수치로 고정**(리팩터 무변경 증명용 기준선):
- 문22: 3-11월 page14 above-gap = 7mm(26.5px)
- 미주사이20 p10: 문10→문12 overflow = 0
- `issue_1082` 4샘플 overflow 현재값, `issue_505`, `issue_1139`
- height_cursor 단위 전체(min-gap 3 + single-line 3 + backtrack 군) 통과 스냅샷

산출물: 필요 시 가드 테스트 추가, `mydocs/working/task_m100_1258_stage1.md`. → 승인.

### Stage 2 — 구현안 결정 (spike)

- render layout 에서 다줄 prev 마지막 줄 trailing 이 y 누적에 빠지는 지점 특정.
- B/A/C 비교 후 택1 + 근거. (코드 변경은 spike 수준, 본 구현은 Stage 3)

산출물: `mydocs/working/task_m100_1258_stage2.md` (선택안 + 근거). → 승인.

### Stage 3 — 선택안 구현 (typeset/render 정규화)

- 택1한 안으로 base-flow trailing 을 단일줄·다줄 일관 처리.
- **이 단계에서는 S8 계열을 아직 제거하지 않음** (정규화만, 동작 동일 확인).
- 검증: 미주 회귀 스위트 전체 green (S8 이 무해하게 공존하는지 확인).

산출물: 소스 + `_stage3.md`. → 승인.

### Stage 4 — S8 계열 축소/제거

- `height_cursor.rs:448-484`(#1246 + #1256/#1261) 분기를 **한 번에 하나씩** 제거, 각 제거 후 전체 미주 회귀.
- 제거 불가(여전히 필요) 분기가 있으면 그대로 두고 사유 기록.
- 배선(`endnote_between_notes_hu`)이 완전 미사용이면 함께 정리, 아니면 유지.

산출물: 소스 + `_stage4.md` (특례 8→N 정량). → 승인.

### Stage 5 — 정리·최종 검증·보고

- 전체 `cargo test` + `cargo clippy --all-targets` green.
- 특례 감소 정량, 위험 영역(C/page-path) 무회귀 확인.
- `mydocs/report/task_m100_1258_report.md` 최종 보고서.

산출물: `_stage5.md` + `_report.md`. → 최종 승인.

---

## 검증 명령 (각 단계 공통)

```
cargo test --lib height_cursor
cargo test --test issue_1082_endnote_multicolumn_drift
cargo test --test issue_505 --test issue_1139_inline_picture_duplicate
cargo test            # 전체
cargo clippy --all-targets
```
+ 문22/미주사이20 시각: `rhwp dump-pages` 또는 export-svg 로 수치 확인.

## 비범위 (불가침)

D 영역(S5/S6/S7), E 영역(S0/S1), `tech-lazy-base-trailing-ls-gate` 게이트 의미, orders 갱신.

## 승인 요청

본 구현계획서 승인 후 Stage 1 착수.
