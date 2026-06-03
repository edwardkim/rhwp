# Task M100-1156 — Stage 1 진단 보고서

- 이슈: [#1156](https://github.com/edwardkim/rhwp/issues/1156)
- 일시: 2026-05-29
- 단계: Stage 1 (진단 + 다단 flow 분석) 완료

## 1. 차트 크기 확정 (80mm, 점유 정상)

- 한컴 + spec(SHAPE_COMPONENT 공통 크기) + HWPX `hp:sz`(22677 HU) = **80mm 3중 일치**
- rhwp 점유: `resolve_object_size`(common 80mm) → 정상
- 차트 그림: MS Graph OLE2 (`ole3.ole` = CFB) → placeholder("OLE 개체 #3") 로 80mm 박스 렌더. 그래프 렌더는 본 task scope 밖 (승인됨).

## 2. 한컴 정답 흐름 (LINE_SEG vpos 역산)

`samples/143E433F503322BD33.hwp` body 895.7px (96dpi, 1px=75HU).

### 단 0
- pi=7 표: vpos=48603, lh=10108 → bottom 782.8px
- pi=9 LINE_SEG: ls0=60911, ls1=62511 (bottom 854.8px) → **단0 used 854.8px** (body 끝까지)
- 차트 80mm(302px): vpos=48603 에 놓으면 bottom **950.4px > body 895.7px** → **단0 초과**
- **한컴: 차트를 단1 로 이동 + 단0 빈 공간(표 아래)에 pi8/pi9 텍스트 back-fill**

### 단 1
- pi=9 ls2 vpos=26644 (리셋, 단1 시작) → pi9 4줄 + pi10~14
- 단1 used 741.4px

## 3. rhwp 결함

```
단 0: used=785px / hwp_used=854.8px (diff -69.6px)
단 1: used=345px / hwp_used=741.4px (diff -396.1px)
```

1. **차트 단 이동 미작동**: 차트가 단0 영역(895px) 초과해도 단0 vpos=48603 에 그대로 (표와 겹침)
2. **텍스트 back-fill 미작동**: 차트가 단0 하단 막아 pi9 가 단1로 일찍 넘어감 → 단1 396px 적게 채움

## 4. 코드 경로 (정정 — main 엔진 = TypesetEngine)

**정정**: 기본 pagination 은 `Paginator`(engine.rs)가 아니라 **`TypesetEngine`(typeset.rs)** (`rendering.rs:1991`, RHWP_USE_PAGINATOR=1 일 때만 engine.rs fallback).

- 실제 pagination: `typeset.rs` `typeset_section_with_variant`
- **근본 원인**: `typeset.rs:1540` `Control::Shape|Picture|Equation => { if !has_table {...} }` — 차트(pi=7)는 표와 같은 문단이라 `has_table=true` → **Shape 처리 블록 스킵** → 차트 높이 가산/단 이동 모두 누락
- 차트 점유 크기: `resolve_object_size` (정상)
- hwp_used: `rendering.rs:3456` `compute_hwp_used_height` (LINE_SEG vpos 권위)

## 4.1 정답지 PDF 시각 확정 (`pdf-large/hwpx/143E433F503322BD33.pdf`, 작업지시자 제공)

한컴 출력: 왼쪽 단(텍스트+표+텍스트, 단 끝까지) / **오른쪽 단 상단에 막대 차트** + 텍스트.
→ 차트가 단0 넘어 단1 상단 이동 + 단0 빈 공간 텍스트 back-fill. LINE_SEG 분석 + 정답지 일치.

## 5. tech 문서

`mydocs/tech/multicolumn_object_flow_backfill.md` — 한컴 알고리즘 + 정정 방향.

## 6. 정정 방향 (Stage 2/3)

- Stage 2: 객체(차트) 현재 단 잔여 < 객체 높이 시 다음 단 이동
- Stage 3: 객체 단 이동 후 후속 문단 빈 공간 back-fill

## 7. 다음 단계

Stage 2 진행 승인 요청.
