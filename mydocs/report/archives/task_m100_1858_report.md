# 최종 결과보고서 — Task #1858

## 이슈

[#1858](https://github.com/edwardkim/rhwp/issues/1858) — 페이지-절대 앵커(vert=용지/쪽)
개체 배치 오차

## 결론

발현 1(3143097 Paper flow 폭발)을 해소했다. `vert=용지` co-anchored 자리차지 상자가 flow 를
소비해 페이지가 폭발하던 것을, 후속 상자도 절대배치(0 flow)하도록 절대배치 가드를 확장하여
한컴 정답지(1쪽)와 일치시켰다. blast-radius 로 collateral 제로 실증. 발현 2(36389312 하단
valign=Bottom 세로 오프셋)는 성숙 코드의 높이/기준 잔여로 회귀 위험이 높아 별도 후속으로 분리.

## 근본 원인 (발현 1)

절대배치 가드(`is_paper_topbottom_block`, typeset.rs ~11827)가 `target_y = host 문단 vpos`
기준 `target_y > current_height` 일 때만 절대배치한다. 같은 host 문단(pi=2)에 co-anchored 된
Paper 상자 22개는 target_y 가 **모두 동일(host vpos)** 이라 첫 상자만 통과하고, 나머지 21개는
current_height 가 이미 sync 되어 가드 실패 → generic flow 경로에서 각자 높이를 소비 → 폭발
(한컴 1쪽 vs rhwp 4쪽, #1853 후 3쪽).

## 수정

같은 host 문단에 선행 Paper 자리차지 표가 있으면(co-anchored 2번째 이후) 후속 상자도
절대배치(0 flow)한다. host 텍스트는 `place_table_with_text` 내 `is_first_placed` 게이트로
중복되지 않는다. 첫 상자는 기존대로 host vpos sync, 단독 Paper 상자는 기존대로 flow.

## 검증

| 항목 | 결과 |
|---|---|
| 3143097 | 4→**1쪽** (한컴 정답지 일치), 상자 22개 전부 1쪽, 시각 렌더 정상 |
| **blast-radius 5,198건** | #1858 추가 변화 3143097(3→1)뿐, 나머지 전부 불변 (collateral 제로) |
| 성숙 앵커/footer 테스트 | issue_1611/1624/1658/1418 무회귀 |
| float 테스트 | issue_1510/1663/1853 무회귀 |
| issue_1858 신규 게이트 | 통과 (구 바이너리 4쪽으로 유효성 확인) |
| lib 2073건 / clippy(lib+tests) | 0 실패 / 0 warning |

## 잔여/후속 — 발현 2 (36389312)

pi=5/pi=6(vert=쪽 valign=Bottom)은 `is_page_bottom_fixed_float` 제외 경로로 배치되어
**페이지 수는 정합(1쪽)** 하나, 하단 발신/결재 블록 전체가 한컴 대비 세로 ~34px 어긋난다
(오라클 현장대응단장 헤딩 y=806.4 vs rhwp ~772). 원인 후보는 pi=6(12×41 발신 grid) 높이
측정 과대 또는 valign=Bottom 기준 좌표. 성숙 코드(#1611/#1658)의 세부라 회귀 위험이 높고
근본 원인 확정에 추가 조사 필요 → 별도 후속(이슈 #1858 잔여 또는 신규 분리).

## 산출물

- 소스: `src/renderer/typeset.rs`
- 게이트: `samples/issue1858_paper_anchor_float_stack.hwpx` + `tests/issue_1858.rs`
- 문서: `plans/task_m100_1858.md`, `working/task_m100_1858_stage1.md`, 본 보고서
