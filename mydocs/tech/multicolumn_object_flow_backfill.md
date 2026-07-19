---
kind: reference
status: active
canonical: mydocs/tech/table_layout_rules.md
last_verified: 2026-07-17
---

# 다단(multicolumn) 객체 단 이동 + 텍스트 back-fill 알고리즘 (한컴 정합)

- 관련 이슈: #1156
- 작성일: 2026-05-29
- 권위: 한컴 LINE_SEG vpos (메모리 룰 `project_hancom_lineseg_behavior`)

## 1. 재현 fixture

`samples/143E433F503322BD33.hwp` / `samples/hwpx/143E433F503322BD33.hwpx`
- 2단(colCount=2) 문서
- pi=7: 표(6×5, wrap=TopAndBottom, tac=true) + **차트(OLE, 80mm, wrap=TopAndBottom)**
- 둘 다 같은 문단(pi=7)의 control

## 2. 한컴 정답 흐름 (LINE_SEG vpos 역산)

좌표 환산: 96dpi, 1px = 75 HU. body 높이 895.7px (67178 HU).

### 단 0 (왼쪽)
한컴 LINE_SEG (pi=9 PartialParagraph):
```
ls[0] vpos=60911   ← 단0
ls[1] vpos=62511   ← 단0 마지막 (bottom 62511+1000+600=64111 HU = 854.8px)
ls[2] vpos=26644   ← vpos 리셋! 단1 시작 (26644 < 62511)
```
- 단0 한컴 used ≈ **854.8px** (body 895.7px 의 끝까지)
- pi=7 표: vpos=48603, lh=10108 → bottom **782.8px**
- pi=8(빈) + pi=9(2줄)이 표 아래 782~854px 에 배치

### 단 1 (오른쪽)
- pi=9 ls[2] vpos=26644 (= 355px, 단 시작 기준) 부터
- pi=9 나머지 4줄 + pi=10~14
- 단1 한컴 used ≈ **741.4px**

### 핵심 — 차트는 단1로 이동
- 차트 80mm = 302px. vpos=48603(648px) 에 놓으면 bottom = 648 + 302 = **950.4px > body 895.7px** → 단0 영역 초과
- **한컴: 차트를 단0에 못 넣고 단1(오른쪽 상단)으로 이동**
- 차트가 비운 단0 하단 공간(표 아래 782~854px)에 **후속 텍스트(pi8 빈줄, pi9 2줄)가 back-fill**

## 3. rhwp 현 동작 (결함)

`dump-pages`:
```
단 0: used=785px / hwp_used=854.8px (diff -69.6px)
단 1: used=345px / hwp_used=741.4px (diff -396.1px)  ← 한컴보다 396px 적게
```

- 차트(pi=7 ci=1)를 **단0 vpos=48603 에 그대로 배치** (표와 겹침)
- 차트가 단0 하단을 막아 pi=9 가 단1로 일찍 넘어감
- 결과: 단1 을 396px 적게 채움 (back-fill 미작동)

### 결함 본질
1. **차트 단 이동 미작동**: 차트가 현재 단(단0) 영역 초과해도 다음 단으로 안 넘김
2. **텍스트 back-fill 미작동**: 차트가 단1로 갔을 때 단0 빈 공간에 후속 문단이 안 채워짐

## 4. 코드 경로 (정정 — 실제 main 엔진은 TypesetEngine)

**중요**: 기본 pagination 엔진은 `Paginator`(engine.rs)가 아니라 **`TypesetEngine`(typeset.rs)** 이다 (`rendering.rs:1991` — "TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback"). engine.rs 는 fallback. 메모리 룰 `feedback_image_renderer_paths_separate` (두 사본).

- 실제 pagination: `src/renderer/typeset.rs` `typeset_section_with_variant`
- **근본 원인**: typeset.rs:1540 `Control::Shape(_) | Picture(_) | Equation(_) => { if !has_table { ... } }`
  - 차트(pi=7)는 같은 문단에 표(Table)가 있어 **`has_table=true`** → **Shape 처리 블록 전체 스킵**
  - → 차트(객체)의 본문 흐름 높이 가산(pushdown, typeset.rs:1621) + 단 이동 모두 처리 안 됨
- 비-TAC TopAndBottom + vert=Para Shape pushdown: typeset.rs:1635 (단, has_table 문단에선 도달 안 함)
- 차트 점유 크기: `resolve_object_size` (common 80mm) — 정상
- hwp_used 추정: `rendering.rs:3456` `compute_hwp_used_height` (LINE_SEG vpos 기준)

## 4.1 정답지 PDF 시각 확인 (`pdf-large/hwpx/143E433F503322BD33.pdf`)

한컴 출력 (정답지):
- **왼쪽 단(단0)**: 텍스트 본문 + 표("연금 재정 전망") + 텍스트가 단 끝까지
- **오른쪽 단(단1)**: **막대 차트(상단)** + 텍스트
- → 차트(80mm)가 단0 끝 넘어 **단1 상단으로 이동**, 단0 빈 공간에 텍스트 back-fill. LINE_SEG 분석과 일치.

## 5. 정정 방향 (Stage 2/3)

### Stage 2 — 차트 단 이동
- 객체(차트/Shape) 배치 시 현재 단 잔여 영역 < 객체 높이(80mm) 면 다음 단으로 이동
- TopAndBottom 객체의 단 경계 판정

### Stage 3 — 텍스트 back-fill
- 객체가 다음 단으로 이동한 뒤, 같은 단의 후속 문단이 객체가 비운 공간을 채우도록 column flow 정정
- 단1 used → hwp_used (741px) 수렴

## 6. 검증 기준

- `dump-pages` 단0/단1 diff 축소 (특히 단1 -396px → ~0)
- 단 경계 LINE_SEG vpos 리셋 지점이 한컴과 일치
- 한컴 한글 2020/2022 시각 판정
