# Task #287 단계 3 완료 보고서 — 시각 비교 + 임시 덤프 제거 + Phase 2 분리

## 타스크

[#287](https://github.com/edwardkim/rhwp/issues/287) — 수식 SVG 레이아웃: 큰 디스플레이 TAC 수식이 줄 상단으로 올라감

- 단계 3 목적: PDF 와 시각 비교, x 정렬 미세 조정, 회귀 재확인, 임시 덤프 제거.

## PDF ↔ SVG 시각 대조

### 박스 내부 큰 수식 `a_{n+1} = { cases ... }`

| 항목 | PDF (`exam_math_8.pdf`) | SVG (수정 후) | 비고 |
|------|------------------------|--------------|------|
| 박스 내부 여부 | ✓ 내부 | ✓ 내부 | 박스 좌상단 겹침 해결 |
| y 위치 (줄 번호) | (가) 줄 다음 | line 1 (y=172.69 + baseline 조정 → eq_y=188.29) | 정상 |
| x 위치 | 수식 `a` 시작 ≈ 162-182 px (박스 x=102 기준 +60-80 px 들여쓰기, 탭 위치 추정) | 133.27 px (박스 기준 +31.2 px) | **차이 약 30-50 px, 왼쪽 치우침** |

### x 차이 원인 진단

덤프 로그(임시):

```
[287-empty] para=0 line=1 col_area.x=71.80 margin_left=30.27 line_indent=31.20 eff_ml=61.47 inline_x=133.27 indent=-31.20 tab_stops=1
```

- `tab_stops=1` 존재. dump 상 `tab[0] pos=13600 HU (=48.0mm=181px)`.
- HWP composed text 에는 "(가) 모든 자연수 n 에 대하여\n**\t**<수식>" 처럼 탭 제어문자가 삽입되어 있고, 탭 위치(181 px)에서 수식이 시작해야 PDF 와 일치.
- 현재 `inline_x = col_area.x + effective_margin_left` 계산은 탭 반영 없음.

### 판단

- **본래 버그(박스 좌상단 겹침, `(71.80, 147.38)` 고정)는 해결됨.** 수식이 박스 내부, line 1 위치에 정상 배치.
- **x 완전 일치는 탭 반영이 필요**하며, compose 계층 수정 포함 가능성 있어 본 타스크 범위 초과로 판단.
- 작업지시자 판단 **"B (현재 상태로 단계 4 진행, Phase 2 별도 이슈)"** 로 결정.

## Phase 2 이슈 등록

- [#288](https://github.com/edwardkim/rhwp/issues/288) — 수식 SVG 레이아웃: 빈 runs 줄의 TAC 수식 x 위치에 탭(\t) 반영 (#287 Phase 2)
- 마일스톤: v1.0.0
- 두 가지 접근 (A: 휴리스틱, B: compose 계층 수정) 후보 기록.

## 임시 덤프 제거

`paragraph_layout.rs` 의 세 지점에 추가했던 환경변수(`RHWP_DUMP_287`) 기반 로그 전부 제거:

1. comp_line 루프 시작부 (단계 1 추가)
2. 인라인 수식 분기 (단계 1 추가)
3. 빈 runs TAC 처리 블록 (단계 3 추가)

커밋 단계에서 원복 확인 필수.

## 회귀 재확인

| 검증 | 결과 |
|------|------|
| `cargo test --lib renderer` | 291 passed, 0 failed |
| `cargo test --test svg_snapshot` | 3 passed |
| `cargo clippy --lib -- -D warnings` | clean |
| 회귀 샘플 diff (단계 2 측정값 유지) | exam_math (20p 중 2p 변경), equation-lim (+0.88 px) — 모두 의도된 개선 |

## 시각적 확인 (수정 후 SVG, 핵심 좌표)

```
# 큰 수식 (수정 후)
<g transform="translate(133.27, 188.29) scale(0.9736,1)">
  ...a _{n+1} = { cases }...
</g>

# 참고: 수정 전 → (71.80, 147.38) 박스 좌상단 고정 (버그)
```

## 산출물

- `mydocs/working/task_m100_287_stage3.md` (본 문서)
- Phase 2 이슈 [#288](https://github.com/edwardkim/rhwp/issues/288)

## 완료 기준 체크

- [x] 큰 수식이 박스 내부 line 1 에 배치
- [x] shape_layout 경로의 중복 렌더 제거
- [x] 기존 회귀 없음 (renderer/svg_snapshot/clippy)
- [x] 임시 덤프 로그 제거
- [x] 남은 개선(x 탭 반영)을 Phase 2 이슈로 분리
- [ ] (단계 4) 최종 보고서 + orders 갱신 + merge 준비

## 승인 요청

단계 3 완료. **단계 4(최종 보고서 작성 + orders 갱신 + merge 준비)** 착수 승인 부탁드립니다.
