# 구현계획서 v2 (Stage 5 추가) — Task #987

> v1 (Stage 1~4) 진행 중 작업지시자 추가 시각 판정으로 발견된
> **쪽 번호 위치** 문제를 Stage 5 로 추가. v1 의 Stage 1~3 은 시각 판정 통과.

## 추가 배경

작업지시자 정정: sample16 쪽 번호는 꼬리말이 아닌 **본문 내 [쪽번호 위치]
컨트롤(PageNumberPos)** 로 정의됨. dump 확인:

```
[2] 새번호: type=Page, number=1
[3] 쪽번호위치: format=0, pos=5   (pos=5 → footer 영역, 가운데 정렬)
```

## 문제 (정량 측정, sample16 page)

| 항목 | y 좌표 (px) |
|------|-------------|
| footer_area | 1046.9 ~ 1084.7 (h=37.8) |
| **현재 쪽 번호 baseline** | **1072.0** (= footer 중앙 + fs/3, `layout.rs:1553`) |
| **쪽 테두리 하변** | **1087.1** (= body_bottom 1068.2 + spacing_b 18.9) |

- 현재: 쪽 번호가 footer_area 중앙(1072) → 쪽 테두리 하변(1087)보다 **위**.
  글자 높이까지 고려하면 테두리 선에 **걸쳐** 보임.
- 한컴 정답지: 쪽 번호가 쪽 테두리 하단 경계선 **아래쪽** footer 여백에 배치.
  (작업지시자: "아래쪽 10.0mm 위가 꼬리말 영역")

## 설계 — 쪽 테두리 하변 기준 배치 (작업지시자 선택)

`build_page_number` 가 `page_border_fill` 을 받아, **쪽 테두리가 존재하고
footer 영역에 쪽 번호를 둘 때** y 를 "쪽 테두리 하변 + 여유" 로 보정.

### 쪽 테두리 하변 공식 (build_page_borders 와 동일)

body 기준: `border_bottom_y = body_area.y + body_area.height + sp_b`
paper 기준: `border_bottom_y = sp_t_from_top ...` (paper 케이스는 기존 로직 재사용)

→ 중복 방지 위해 "쪽 테두리 하변 y" 산출을 헬퍼로 추출하거나,
  build_page_borders 와 동일 계산식을 build_page_number 에서 재현.
  (구현 시 헬퍼 추출 우선 검토 — DRY)

### 변경 지점

1. `src/renderer/layout.rs:628` 호출부
   — `build_page_number(.., page_border_fill, ..)` 인자 추가
2. `src/renderer/layout.rs:1497` `build_page_number` 시그니처
   — `page_border_fill: Option<&PageBorderFill>` 추가
3. `src/renderer/layout.rs:1553` y 계산
   — footer position 이고 page_border_fill 존재 시:
     `y = border_bottom_y + gap + font_baseline`
   - gap: 쪽 테두리 하변과 쪽 번호 사이 여유 (한컴 실측으로 확정)
   - page_border_fill 없으면 기존 footer 중앙 로직 유지 (회귀 방지)

### 격리 원칙

- page_border_fill 이 **없는** 문서(시험지 등 다수) 는 기존 동작 그대로.
  → 쪽 테두리 있는 문서만 영향. HWP3/HWP5/HWPX 공통 경로지만
    "쪽 테두리 존재" 조건으로 자연 격리.

## Stage 5 단계

1. `build_page_number` 에 page_border_fill 전달 + 쪽 테두리 하변 y 산출
2. footer position + 쪽 테두리 존재 시 y 보정, gap 값 시각 판정으로 확정
3. 검증: sample16 다중 페이지 + 쪽 테두리 없는 샘플 회귀
4. 작업지시자 시각 판정

## 검증 계획

- sample16 p0/p1/p2 쪽 번호가 쪽 테두리 하변 아래 배치 (시각 판정)
- 쪽 테두리 **없는** 문서: 쪽 번호 위치 불변 (회귀 없음)
- `cargo test` / `cargo clippy -- -D warnings`

## 산출물

- `mydocs/working/task_m100_987_stage5.md`
- 최종: `mydocs/report/task_m100_987_report.md` (Stage 1~5 통합)

## 승인 요청

Stage 5 (쪽 테두리 하변 기준 쪽 번호 배치) 구현 방향 승인 요청드립니다.
gap(테두리~쪽번호 여유) 구체값은 1차 구현 후 시각 판정으로 확정하겠습니다.
