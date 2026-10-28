# Task #4639 Stage 1 — TAC host 의 동거 개체 float 밴드를 사다리 스냅으로 회복

Issue: #4639 (#4599 캠페인 Phase 3, 사이클 4)
Branch: `fix/4599-tac-host-sibling-float-snap` (base: devel 572786d02)

## 무엇을

treat_as_char(블록) 표를 품은 문단 직후에는 저장 사다리 보정(vpos_adjust)을 건너뛰는
규칙(prev_tac_seg_applied — TAC 전진 경로가 다음 흐름을 이미 확정한다는 가정)이 있다.
그런데 그 host 문단이 **비-TAC 개체 float(그림 등, TopAndBottom/Square·PARA 상대)**를
함께 앵커하면, TAC 전진은 표 줄만 소비하고 개체 밴드는 흐름에 남는다 — 후속 문단이
개체 위에 겹쳐 렌더된다. 이 형상에서만 스킵을 해제해 저장 사다리 보정이 후속 문단을
개체 아래로 스냅하게 했다. hwpx stored layout 한정.

## 실측 근거 (36442008 p1 — #4599 uniform − 군집 대표, 220.8px)

- pi5 = TAC 8×9 표(477.2..688.6) + **전폭 SQUARE 그림(pic, vertOffset=17207HU →
  706.7..911.3)** 동거 host. TAC 전진 213.2px 뿐 → pi8 '붙임'이 734.2 로 그림 위에
  겹침.
- 한글 2022 캐시 PDF: '붙임' 955.2. 저장 사다리: pi6 vpos 62682 → 911.4(그림 하단),
  pi8 → 955.0 — **사다리·PDF 일치**(이 문서군의 사다리는 신뢰 가능).
- 수정 후: pi6 911.3 · pi8 955.1 (PDF 와 0.1px) — **CONFIRMED(220.8) → QUIET(0.5)**,
  페이지 전체 n=46 쌍 maxdev 0.5.

주: 그림이 TopAndBottom 이 아니라 **Square(어울림) 전폭**임을 XML 로 확정하고 게이트에
Square 를 포함시켰다(전폭 Square = 사실상 상하 배치). 스냅은 저장 좌표를 따르므로
옆-흐름 Square 문서에서도 사다리가 진실을 말한다.

## 검증 실측

### #4599 backlog 242 재판정 — 개선 1(220.8→0.5 QUIET) / 불변 241 / 악화 0

### hwpx 3418 스윕 (baseline 572786d02 클린 worktree 빌드)

- 판정 이동 1건: 대상 36442008 DRIFT→OK (worst 220.8→0.1)
- worst +2px 악화 0건 · 전수 Table-diff 변동 0건

### 저장소 게이트

- cargo fmt --check: 통과 · clippy: 경고 0 · cargo test --release 전체: exit 0 ·
  wasm-pack build --target web: 성공
