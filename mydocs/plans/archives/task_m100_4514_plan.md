# Task #4514 수행 계획 — overlay 표 필러 흐름 유실 수정

Issue: #4514 (표가 페이지 경계에서 분할되지 않고 겹쳐 렌더링됨 — 47쪽 중 6쪽).
재현: `samples/issue4514/sample1-repro.hwp` (#4515 브랜치에서 편입).

## 원인 규명 (devel HEAD 8ea92cdad 실측)

문서 저작 패턴: 4×5 요구사항 표 37개가 `treat_as_char=false, vert=문단` 으로 **빈 문단에
앵커**되고(글앞으로 ~29개 + 자리차지 ~8개), 각 host 뒤 빈 문단 ~15개(각 2400HWPU)가
표 높이만큼 흐름 공간을 만든다. 저장 사다리: host 0.102(vpos 7528) → 필러 103~117
(9768…42888, 각 자기 줄 높이만큼 전진) → 다음 host 0.118(vpos 45288 = 117 끝과 정확히
연속).

주 페이지네이션(TypesetEngine, typeset.rs)에서 두 메커니즘이 결합해 이 공간이 소멸한다.

1. **#703 Shape 단축** (typeset.rs ~16586): 단일 컬럼의 비-TAC 글앞/글뒤 표는
   `oversized_multirow`(본문 높이 초과)가 아니면 `PageItem::Shape` 로 배치되어 흐름
   공간을 차지하지 않는다. 문제 표들(494~617px < 본문 971px)은 전부 이 분기다.
2. **#1955 후행 빈 문단 흡수** (typeset.rs ~6591, `behind_float_table_para`): overlay
   비-TAC 표 anchor 직후의 빈 문단들을 흐름 소비 없이 anchor 단에 소급 흡수한다.
   도입 사유는 "현재 페이지네이션은 fragment 로 플로우를 소비하므로" — 즉 **표가
   fragment 로 흐름을 이미 소비한 경우**(조례 [별표] 같은 oversized 표)의 이중 계상
   방지다.

Shape 단축(흐름 0) + 흡수(흐름 0)가 동시에 걸리면 사다리 갭(≈표 높이)이 어느 쪽에도
계상되지 않는다. 실측(`RHWP_TABLE_DRIFT`): 8쪽 항목열 99→100→101→102→118→119→139→143,
필러 전무, overlay host 항목 전진 0. 렌더러 `vpos_adjust`(height_cursor.rs)의 사다리
보정은 **직전 문단의 끝 vpos** 기준이라 흡수로 사라진 갭(102 끝 9768 → 118 시작 45288)
을 복원하지 못한다. vert=문단 앵커라 표 y 는 host 문단 y 를 따라가 연쇄 겹침이 된다.

정상 페이지가 다수인 이유: 표 1개/쪽 구성에서는 쪽 경계 리셋·후속 항목의 저장 vpos
스냅이 우연히 위치를 복원하기 때문. 표가 연속되고 쪽나눔 위치가 어긋나는 구간(8·12·13·
22·25·29쪽)에서만 국소 붕괴 — 이슈의 "길고 표가 연속되는 문서에 집중" 관찰과 일치.

참고: 폴백 페이지네이터(`RHWP_USE_PAGINATOR=1`)는 더 나쁨(43쪽, 겹침 10건+) — 수정
대상은 TypesetEngine 이다. PR #4520(overlay+TAC·Picture/Shape host)은 이 케이스를
커버하지 않음을 실측으로 확인(47쪽 render tree 바이트 동일).

## 수정 설계

**흡수 arming 을 "표가 실제로 흐름을 소비한 경우"로 한정한다.**

- TypesetState 에 `overlay_shape_shortcut_para: Option<usize>` 추가.
- #703 Shape 단축 분기(16596)에서 `Some(para_idx)` 기록.
- #1955 arming(6938~6947)에서 `overlay_shape_shortcut_para == Some(para_idx)` 면 arming
  생략 → 후행 빈 문단들이 정상 흐름(자기 줄 높이 전진)으로 남는다.

기대 효과: 필러가 흐름을 만들면 (a) 사다리 갭이 자연 복원되고 (b) 페이지 fit 이
한컴과 같은 곳에서 끊기며 (c) 자리차지 표(118)는 기존 RowBreak fragment 기제로 쪽
경계 분할된다. 한컴 계약: ECR-001~005 구간 3쪽, 총 46쪽.

한계/비대상: 한 문단에 shortcut 표와 fragment 표가 공존하는 극단 케이스는 arming 을
생략한다(이중 공간 위험보다 드묾 — 주석으로 명시). oversized fragment 경로(#1955 원
사례)는 종전대로 흡수 유지.

## 검증 계획

1. sample1-repro: `LAYOUT_TABLE_OVERLAP` 0건(#4515 진단·통합 테스트 자기일관 유지),
   총 페이지 수 46(한컴), ECR-001~005 3쪽, 8쪽 사다리 정합(118 시작 ≈ 본문 604px).
2. 시각 증적: 수정 전후 8·12·13·22·25·29쪽 PNG/SVG.
3. 회귀: #1955 조례 [별표] 관련 기존 테스트, behind_float 관련 골든, release-test 전체,
   Native Skia 3종 (승인 완료된 게이트).
4. render-diff 10k 서베이급 광범위 회귀는 별도 승인 시.
