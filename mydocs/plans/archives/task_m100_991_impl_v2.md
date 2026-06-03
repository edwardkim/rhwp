# 구현계획서 v2 — 페이지 분할된 셀의 줄 중복 렌더링

- 타스크: 로컬 task991
- 브랜치: `local/task991`
- 수행계획서: `task_m100_991_v2.md`
- v1: `task_m100_991_impl.md` (전제 오류로 폐기)
- 작성일: 2026-05-19

## 수정 대상

`src/renderer/layout/table_layout.rs` — `compute_cell_line_ranges` 의 일반 문단 줄 단위 루프(원자 문단 경로 아님).

현재 분할 시작 페이지(offset)의 줄 스킵 판정:

```rust
if has_offset && line_end_pos <= content_offset {  // line_end_pos = cum + line_h
    cum = line_end_pos;
    para_start = li + 1;
    para_end = li + 1;
    continue;
}
```

분할 끝 페이지(limit)의 줄 포함 판정은 `line_break_pos = cum + h` (줄간격 제외) 를 쓴다.

## 수정 방향

offset 스킵 판정을 limit 판정과 동일한 `line_break_pos = cum + h` 기준으로 일치화한다.

```rust
let line_break_pos = cum + h;   // limit/offset 공통 기준으로 끌어올림
if has_offset && line_break_pos <= content_offset {
    cum = line_end_pos;          // cum 누적은 종전대로 line_end_pos
    para_start = li + 1;
    para_end = li + 1;
    continue;
}
```

근거: 분할 끝 페이지 `abs_limit` 와 분할 시작 페이지 `content_offset` 은 동일한 분할 경계값이다. 양쪽이 같은 `cum+h` 기준을 쓰면 — 끝 페이지 포함 조건 `cum+h ≤ 경계` 와 시작 페이지 스킵 조건 `cum+h ≤ 경계` 가 정확히 상보(complement)가 되어, 모든 줄이 정확히 한 페이지에만 배치된다(중복·누락 모두 없음). `cum` 누적값은 두 호출에서 동일하므로 줄별 `cum+h` 도 동일하다.

원자 문단 경로(`para_end_pos` 사용)는 limit·offset 양쪽이 이미 동일 기준이므로 손대지 않는다.

## 단계

### 2단계 — 수정 구현

- 위 한 줄(스킵 조건식) 일치화 + `line_break_pos` 계산 위치 조정.
- 변경 의도를 설명하는 주석 추가(Task #656 후속 — offset 측 기준 일치).
- 비공개 샘플 `export-svg` 로 cp[1] 줄이 6·7쪽 중 한 쪽에만 나타나는지 확인.
- `dump-pages` 로 분할 행 높이/항목이 합리적인지 확인.
- 산출물: `task_m100_991_stage2.md` + 소스 커밋.

### 3단계 — 검증 및 보고

- `cargo build` / `cargo test` 전체 통과, `cargo clippy` 경고 없음.
- 골든 SVG 테스트 회귀 없음 — 회귀 발생 시 각 건이 정정인지 퇴행인지 판정.
- 다른 공개 분할 표 샘플로 교차 회귀 확인.
- 회귀 방지 테스트: 공개 샘플 중 분할 셀 케이스를 찾아 추가하거나, 불가 시 비커밋 처리 후 보고서 명시.
- WASM 영향 시 Docker 재빌드.
- 산출물: `task_m100_991_stage3.md` + `report/task_m100_991_report.md` + `orders/20260519.md`.

## 회귀 리스크 점검

- `cum+h` 로 스킵 기준을 강화하면(`line_h ≥ h` 이므로 종전보다 더 많은 줄을 "이전 페이지 소속"으로 스킵하지 않고 **덜** 스킵 → 시작 페이지에 줄이 더 포함될 수 있음). 그러나 끝 페이지가 같은 기준으로 이미 그 줄을 포함했다면 중복이 되고, 포함하지 않았다면 정상이다. 핵심은 끝 페이지와의 상보성이며, 두 측이 동일 기준이면 정의상 중복·누락이 불가능하다.
- 골든 SVG 테스트로 기존 분할 표 케이스 회귀를 전수 확인한다.
