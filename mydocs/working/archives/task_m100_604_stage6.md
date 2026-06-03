# Task #604 Stage 6 — paper_images z-order 정합 (한컴 변환 시각 정합 완성)

## 본 단계 본질

Stage 5 (옵션 B-2) 의 lh 정합화 후 추가 검증 — 본문이 cs/sw=정합 wrap zone 안에 표시
되지만 일부 라인이 그림 영역 (sequential flow y) 에 침범. 결함의 본질 = paper_images
z-order 가 body_node 보다 아래 layer.

## 결함 본질 진단

`src/renderer/layout.rs:491-496` (Stage 6 정정 이전):
```rust
// 용지 기준 이미지: body clip 바깥에 배치 (배경 이미지 등)
for img_node in paper_images {
    tree.root.children.push(img_node);  // ← bottom layer
}
tree.root.children.push(body_node);     // ← top layer
```

z-order 본질:
- `paper_images` 가 body_node 보다 **먼저** 추가 → SVG paint order 상 그림이 본문 아래
- 본문 sequential flow y 가 그림 영역까지 흐르면 그림이 가리지 못함 → 본문이 그림 위에
  보임 → 시각 결함

## 정정 — z-order 변경 (그림을 top layer 로)

```rust
tree.root.children.push(body_node);          // ← bottom layer (sequential flow)

// 한컴 변환 메커니즘 정합: paper_images 를 body 위 z-layer 로
for img_node in paper_images {
    tree.root.children.push(img_node);       // ← top layer (그림이 본문 가림)
}
```

**한컴 변환 메커니즘 본질** (3 요소 모두 정합):
1. wrap zone 안 라인의 lh=th (line_spacing 100%) — Stage 5 B-2
2. 그림 = paper-relative absolute layer (vert/horz_rel_to=Paper) — 기존 메커니즘
3. **그림이 본문 위에 그려져 가림** — Stage 6 정정 ★

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo test` 통합 31 | ✅ 모두 통과 |
| `cargo test --test issue_546` | ✅ exam_science 4페이지 / items=37 |
| `cargo test --test issue_554` | ✅ 12 passed |

### 회귀 영역
- HWP3 native: hwp3-sample.hwp 16p / hwp3-sample5.hwp 64p / hwp3-sample4.hwp 40p (회귀 0)

## 시각 판정 자료

`output/svg/task604_stage6_b/hwp3-sample5/hwp3-sample5_{004,008,016,022,027}.svg`

본 정정 효과 (사용자 시각 판정):
- pi=75 wrap text 가 그림 우측 좁은 영역에 lh=900 정합 표시 (Stage 5 B-2 효과)
- 그림이 본문 위 layer 에 그려져 본문 침범 시각 결함 해소 (Stage 6 효과)
- HWP5 v2018/v2024 변환본과 거의 시각 정합 ★

## LOC 합계

| 파일 | 변경 |
|------|-----|
| `src/renderer/layout.rs` | +5/-3 (z-order 변경 + 주석) |
| **소스 합계** | **+2 LOC** |

## 한컴 변환 메커니즘 정합 본질 (Stage 5 B-2 + Stage 6 결합)

본 두 정정으로 한컴 v2018/v2024 변환본의 시각 본질을 모두 모방:

| 요소 | Stage | 본질 |
|------|-------|------|
| LineSeg cs/sw 정확 인코딩 | Stage 3 | wrap zone 위치 정합 |
| LineSeg lh 100% 줄간격 | Stage 5 B-2 | 좁은 wrap zone 안 글자 수용 |
| 그림 paper-relative absolute layer | (기존) | sequential flow 와 분리 |
| 그림 top z-layer | Stage 6 | 본문 침범 시 시각 가림 |

## 작업지시자 승인 요청

본 Stage 6 정정 완료 보고. 광범위 회귀 검증 + 시각 판정 + 최종 보고서 갱신 + 재PR 진행
승인 요청.

## 참조

- Stage 1~5 보고서: `mydocs/working/task_m100_604_stage{1,2,2b,3,5}.md`
- Stage 4 (재) commit: `9ca9ce2` (drop)
- Stage 5 (B-2) commit: `fdb21a2`
- Stage 6 commit: (본 commit)
- 최종 보고서: `mydocs/report/task_m100_604_report.md`
- Issue #604
