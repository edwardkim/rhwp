# Task #4650 Stage 1 — Square 표 옆 wrap prefix 분리를 다줄 전폭 꼬리로 확장

Issue: #4650 (#4599 캠페인 Phase 3, 사이클 6)
Branch: `fix/4599-square-wrap-prefix-split` (base: devel 572786d02)

## 무엇을

반폭 Square(어울림) 표 옆으로 흐르다 전폭으로 복귀하는 문단의 wrap prefix 분리
(#4090)가 "전폭 꼬리 정확히 한 줄"에만 적용되어, 꼬리가 여러 줄인 문단이 일반 배치로
떨어지며 prefix 까지 표 하단 아래로 밀렸다. 분리 판별을 "꼬리 전 줄 전폭 + 저장
seg·조판 줄 1:1"로 확장했다 — prefix 는 wrap 띠(표 옆), 꼬리는 표 아래 전폭.

- `src/renderer/typeset.rs` — `can_split_prefix` 판별 확장 (꼬리 1줄 → 전-전폭 꼬리)

## 실측 근거 (156714641 p1 — #4599 single+/STEP 군집 대표, 204.0px)

- pi12 = 반폭(340.7px) Square 2×1 표. pi13 = 9 seg 문단 — 저장 seg 0..4 가
  cs=25835·sw=22353(표 옆 우측 절반), 5..8 이 전폭. 저장 사다리가 wrap 형상을 증언.
- 한글 2022 캐시 PDF: '연구진은…'(prefix 첫 줄) 747.9 — 표 옆.
- 종전 rhwp: #4090 게이트(꼬리 1줄) 불발 → 일반 배치 → Table 아이템이 흐름을 표
  하단(952.9)까지 전진시킨 뒤 prefix 를 그 아래에 렌더 (+204.0px).
- 수정 후: prefix 표 옆, 판정 **CONFIRMED(204.0) → WEAK(9.2)** (매칭 쌍 9→14,
  잔여는 좁은 seg 의 줄바꿈 미세 차이).

## 검증 실측 (단독 브랜치, base 572786d02)

### #4599 backlog 242 재판정 — 개선 1(204.0→9.2 WEAK) / 불변 241 / 악화 0

### hwpx 3418 스윕 (baseline 572786d02 클린 worktree 빌드)

- 판정 이동 1건: 대상 DRIFT→OK (worst 198.9→10.2)
- worst +2px 이동 1건: 156492236 +7.5 (30.9→38.4) — **모아찍기(NUP_PDF) 캐시 문서**로
  오라클 부재. 같은 반폭 Square wrap 템플릿이 분리 경로에 진입하며 사다리 축 ±16px
  이동(표 이동 포함). Windows 캐시 재생성(기존 목록 등재) 후 재판정 대상으로 기록.
- 전수 Table-diff 변동 2: 대상(의도) · 156492236(위)

### 저장소 게이트

- cargo fmt --check: 통과 · clippy: 경고 0 · cargo test --release 전체: exit 0 ·
  wasm-pack build --target web: 성공
