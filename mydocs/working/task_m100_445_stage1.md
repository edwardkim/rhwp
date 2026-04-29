# Task #445 Stage 1 — 원인 확정 (진단)

**브랜치**: `local/task445`
**작성일**: 2026-04-29

---

## 1. 진단 방법

`paragraph_layout.rs:2535` push 직전과 `layout.rs:1614` merge 입력/출력에 임시 `eprintln!` 삽입.

```rust
// paragraph_layout.rs (push 직전)
eprintln!("TASK445_RANGE: pi={} bf={} y_start={:.1} y_end={:.1} h={:.1} sl={} el={} comp_lines={} ps={} pe={}", ...);

// layout.rs (merge 전후)
eprintln!("TASK445_MERGE_INPUT: ranges_count={}", ranges.len());
eprintln!("TASK445_MERGE_OUTPUT: groups_count={}", groups.len());
```

`./target/release/rhwp export-svg samples/exam_kor.hwp -p 7` 실행 후 stderr 분석.

## 2. 핵심 발견

### 2.1 비정상 push: page 8 col 0 의 마지막 paragraph

```
TASK445_RANGE: pi=298 bf=7 y_start=1546.7 y_end=1669.3 h=122.5
                sl=0 el=5 comp_lines=13 ps=false pe=true
```

- `pi=298` 의 `y_start=1546.7` 이 이미 col_bottom(1423) 을 **123px 초과**
- y_end=1669.3 (페이지 바닥 1587 도 초과)
- start_line=0, end_line=5, total composed lines=13 → 5줄 PartialParagraph 의 컬럼 끝부분

### 2.2 직전 paragraph 의 layout 높이 vs 페이지네이션 추정 불일치

```
TASK445_RANGE: pi=297 bf=7 y_start=1286.3 y_end=1546.7 h=260.4 sl=0 el=11
```

- pagination 의 `dump-pages` 보고: pi=297 `h=168.7 (lines=168.7)`, `vpos=80602..9190 [vpos-reset@line5]`
- 실제 layout: 11줄 모두 선형 누적 → 260.4px (페이지네이션 추정 대비 **+91.7px 오버슈트**)

### 2.3 근본 원인: vpos-reset 미존중

- HWP 의 `vertical_pos == 0` (line>0) 은 해당 줄에서 vpos 가 리셋됨을 표시. 일반적으로 다단/페이지 분할 시점.
- `pagination/engine.rs:597`: `respect_vpos_reset` 옵션이 활성화될 때만 forced_breaks 처리
- `main.rs:111`: 기본값 `false` (CLI flag 로만 활성화)
- 결과: pi=297 의 line 5 이후가 col 0 에 그대로 누적 → pi=298 시작점 이미 col_bottom 너머

### 2.4 expand_clip 의 부수적 증상

`layout.rs:442-474` `expand_clip()` 는 자식 bbox 가 body_area 를 초과하면 body-clip 을 확장만 함. paragraph border 의 bbox 가 1669 까지 가면 clip 도 1671 까지 확장 → SVG body-clip h=1474 (페이지 1587 대비 84px 부족하지만 매우 큼).

## 3. 가설 검증

| 가설 | 결과 |
|------|------|
| (A) PartialParagraph 의 bbox 자체가 잘못됨 | ✅ 부분 적중 (분할 전 layout y_start 가 잘못된 위치) |
| (B) border 그리기 단계 y 계산 오류 | ❌ y 계산은 정상. start_line..end 만 순회하여 누적 |
| (C) expand_clip 의 정책 부적절 | △ 증상 증폭 요인. 근본은 아님 |

**확정**: 페이지네이션의 vpos-reset 미존중으로 인해 pi=297 같이 vpos-reset 가 있는 paragraph 가 FullParagraph 로 처리되어, 후속 PartialParagraph 가 col_bottom 너머에서 layout 됨. 이때 그 paragraph 의 border 가 함께 그려져 페이지 바깥까지 침범.

## 4. 수정 범위 결정

**근본 수정 (vpos-reset 기본 활성화)** 은 다른 회귀 위험이 큼 (`main.rs:111` 기본값 변경 + 영향 범위 광범위). Task #445 범위를 초과.

**좁은 범위 수정**: paragraph border merge 결과를 col_area 바닥으로 클램프. 텍스트 자체의 overflow 처리는 별도 이슈로 남김.

- 대상: `layout.rs:1635` 직후 (groups 병합 완료 시점)
- 방식:
  ```rust
  let col_top = col_area.y;
  let col_bot = col_area.y + col_area.height;
  for g in groups.iter_mut() {
      if g.2 < col_top { g.2 = col_top; }
      if g.4 > col_bot { g.4 = col_bot; }
  }
  groups.retain(|g| g.4 > g.2);
  ```

## 5. 산출물

- 임시 진단 코드 삽입/제거 (Stage 2 에서 정리)
- 본 보고서

다음 단계: Stage 2 — 좁은 범위 수정 적용 + 테스트.
