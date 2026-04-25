# Task #291 Stage 1 — 조사·실증

## 목표

KTX.hwp 2단 구성 TAC 표 좌측 밀림 증상의 근본 원인 식별.

## 트러블슈팅 사전 검색 (memory 규칙)

| 문서 | 관련성 |
|------|--------|
| `column_def_proportional_widths.md` (2026-02-16) | 비례값 인코딩 — 이미 해결, 본 이슈와 무관 |
| `multi_tac_table_pagination.md` | 다중 TAC 표 페이지네이션 — 본 이슈와 다름 |
| `tab_tac_overlap_142_159.md` | 탭/TAC 겹침 — 본 이슈와 다름 |

→ 신규 이슈로 처리. 트러블슈팅에 등록 가치 있음 (Stage 4).

## 좌표 측정

### 페이지 레이아웃 (96dpi)

| 항목 | mm | px |
|------|----|----|
| 용지 | 297 × 210 (가로) | - |
| 좌우 여백 | 8mm | - |
| body 너비 | 281mm | 1062.0 |
| 단 0 (왼쪽) | 117.7mm | 444.85 |
| 단 간격 | 5.1mm | 19.28 |
| 단 1 (오른쪽) | 158.3mm | 598.30 |

### 단 영역 좌표 (기대값)

- 단 0: x=30.24 ~ 475.09
- 단 1: x=494.37 ~ 1092.67

### 표 좌측 시작 x 측정

| 표 | 좌측 x | 라벨 위치 | 평가 |
|----|--------|-----------|------|
| **pi=29** (3x11+3x9, 비-TAC) | **744.71** | x=529 | ✅ 한컴과 유사 |
| pi=30 ci=1 (1x2, 비-TAC) | (340 폭) | x=496 | - |
| **pi=31** (16x15, **TAC**) | **494.10** | x=496 | ❌ 단 좌측 밀림 |
| **pi=32** (10x9, **TAC**) | **494.10** | x=496 | ❌ 단 좌측 밀림 |

### ParaShape 비교

| 표 | TAC | wrap | horz | ParaShape align | tab_def 마지막 |
|----|-----|------|------|-----------------|----------------|
| pi=29 | false | 위아래 | **단(8.8mm)** ← 안쪽 들여쓰기 | Right | tab pos=2 (0mm) |
| pi=31 | **true** | 위아래 | **문단(0mm)** ← 좌측 붙음 | **Right** | tab pos=56762 (200.2mm) |
| pi=32 | **true** | 위아래 | **문단(0mm)** | **Right** | tab pos=56762 (200.2mm) |

**핵심 차이**:
- 비-TAC (pi=29): `t.common.horz_rel_to=Column, horizontal_offset=8.8mm` 으로 표가 단 안에 배치됨 → 한컴과 유사
- TAC (pi=31/32): `t.common.horz_rel_to=Para, horizontal_offset=0mm` 인데 ParaShape `align=Right` → 한컴은 단 우측 정렬, rhwp 는 좌측 붙임

## 한컴 기대 위치 계산

- 단 1 시작: 494.37
- 단 1 끝: 1092.67
- 표 폭 (pi=31): 574.11
- 한컴 기대 좌측 x = 1092.67 - 574.11 = **518.56**

현재 위치 494.10 vs 기대 518.56 = **24.46px (6.5mm) 차이**

## 코드 위치 식별

`src/renderer/layout.rs:1991~2003` — TAC 표 분기 (`tbl_inline_x` 계산):

```rust
} else if is_tac {
    let leading = ...;
    Some(col_area.x + effective_margin + leading)  // ← 좌측 정렬 강제
}
```

비-TAC + Square wrap 분기 (Task #295) 에는 `t.common.horz_align` 반영이 있으나 **TAC 분기는 ParaShape `alignment` 무시**.

## 결론

### 근본 원인
TAC 표 분기에서 ParaShape `alignment` 반영 누락. `align=Right/Center` 인 TAC 표가 단 좌측에 강제 정렬됨.

### 수정 방향
TAC 분기에 alignment 매치 분기 추가:
- Right → 단 우측 끝 - 표 폭
- Center → 단 중앙
- 기타 → 기존 base_x

### 검증 방향
1. KTX.hwp pi=31/32 좌측 x: 494 → 518 ± 1
2. cargo test 회귀 0
3. 5샘플 byte-diff (개선/회귀 분리)
4. 브라우저 시각 검증
