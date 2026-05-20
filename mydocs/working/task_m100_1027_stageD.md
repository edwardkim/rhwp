# Stage D 조사보고서 — #1027: 페이지네이터 vpos 스냅 통합 (커밋 보류)

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage D — HeightCursor 페이지네이터 통합 시도 + 신규 overflow 진단
- 상태: **커밋 보류** (작업지시자 지시: "커밋 보류 + 더 조사")
- 검증 샘플: `samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx` (184p, 비공개)

## 1. 구현 (커밋 안 함, 작업트리 보존)

`typeset.rs`: TypesetState 에 vpos 스냅 상태(page/lazy base, prev, anchor) 추가, 컬럼
경계 reset, `vpos_snap_current_height()` 헬퍼로 항목 fit 직전 `current_height` 를
vpos-정합 위치로 스냅(Stage C `HeightCursor::vpos_adjust` 호출), 항목 후 prev/base 추적.
누적식 `+= total_height` 는 유지(증분만 보정). 단단 전용.

## 2. 결과 (Stage C vs Stage D)

| 항목 | Stage C | Stage D |
|------|---------|---------|
| 노트 "추진일정은"(pi=127) | **9쪽**(오류) | **8쪽**(한컴 PDF 정합 ✓) |
| 페이지 수 | 185 | 184 |
| svg_snapshot(공개) | 5 pass / 3 debt | 5 pass / 3 debt (신규 회귀 0) |
| lib 테스트 | 1316 | 1316 |
| LAYOUT_OVERFLOW | 13 | **18 (+5)** |

**신규 overflow 5건**: p6/120(6.6), p7/129(6.6), p8/143(4.2 — 노트 영역), p78/429(5.1),
**p71/361(23.4 — TAC 표 직후)**. (기존 다수는 페이지 -1 시프트일 뿐 값 동일.)

## 3. 근본 원인 진단 (p71 = 72쪽, 실렌더 VPOS_CORR 로그)

페이지 72: TAC 표 pi=349 후 단락 pi=350~361. 렌더러 실측:

- pi=347·348·349: `path=page` 보정 적용(early items 백워드 보정 성공).
- pi=349 TAC 표 후 base 무효화 → pi=350~ `path=lazy`.
- **pi=354 부터 모두 `applied=false`**: 보정 목표 end_y 가 현재 y_in 보다 8px 넘게 위 →
  `MAX_BACKWARD_PX=8` 클램프가 보정 전면 거부.
- 렌더러 y 가 vpos-정답 대비 **전방 드리프트 누적**:
  - pi=357 y_in=634.2 vs end_y=606.2 (Δ28)
  - pi=361 y_in=1000.4 vs end_y=956.4 (**Δ44**) → pi=361 경계(1046.9) 밖 1070.4 렌더 → 23.5px 초과.

**핵심**: 렌더러 자신이 8px 백워드 클램프 때문에 vpos 를 완전히 추종하지 못하고
**페이지당 최대 ~44px 전방 드리프트**한다. 페이지네이터(Stage D 스냅)는 vpos 를 가깝게
추종(드리프트 제거)하므로, **드리프트하는 렌더러와 어긋나** fit 을 과대 판정 → overflow.

## 4. 왜 Stage C 는 p71 에서 overflow 안 났나

Stage C 의 `+= total_height` 누적은 렌더러의 전방 드리프트와 **우연히 비슷한 양**으로
과측정 → p71 에선 렌더러와 정합(overflow 없음). 그러나 같은 과측정이 노트 페이지에선
43.6px 과측정으로 노트를 9쪽으로 밀어냄. 즉 **단일 누적 정책으로는 페이지마다 다른
렌더러 드리프트를 동시에 맞출 수 없음**(Stage 3 결론 재확인).

## 5. 진짜 정합 조건

노트(8쪽)와 무-overflow 를 동시에 만족하려면 **렌더러와 페이지네이터의 y-진행이 항목별로
동일**해야 한다. Stage D 는 inter-item vpos 스냅만 공유했고, 두 측정의 어긋남(특히 TAC 표
host advance 후 lazy base 가 각자의 누적에서 독립 역산되어 값이 갈림)은 남는다.

선택지:
- **(A) 렌더러 백워드 클램프 완화**: 렌더러가 vpos 를 완전 추종(드리프트 0)하게 하면
  페이지네이터 스냅과 자동 정합. #643/#874 가 8px 로 제한한 이유(불신뢰 vpos forward
  jump 가드)를 해치지 않는 범위 확인 필요. → 노트 + overflow 동시 해결 가능성 가장 높음.
- **(B) per-item advance 완전 공유**: 렌더러 layout_column_item 의 y-math(특히 TAC 표
  host)를 height-only 함수로 추출해 페이지네이터가 동일 호출. 두 누적이 절대 갈리지 않음.
  규모 큼(Stage E~F).

## 6. 가설 (A) 실험 결과 — 렌더러 백워드 클램프 완화 (임시, 되돌림)

Stage D 스냅 병행 + `MAX_BACKWARD_PX` 스윕 (AI 184p):

| max_backward | 페이지 | overflow | 노트 |
|--------------|--------|----------|------|
| 8 (기본) | 184 | 18 | 8쪽 |
| 50 | 183 | 16 | 8쪽 |
| 200 | 183 | 16 | 8쪽 |
| 2000 | 177 | **17** | 8쪽 |

- 완화는 overflow 18→16 으로 **부분 개선**(baseline 13 미달).
- 과도 완화(2000)는 페이지 177 로 급감(렌더러 과압축) + overflow 재증가 → 부작용 큼.
- 결론: **(A) 단독 불가**. 페이지네이터·렌더러가 TAC 표 host advance / lazy base 를 각자
  독립 산출해 어긋나는 게 근본 → **(B) per-item advance 완전 공유 필수**.

## 7. 수술적 조사 — p71 23.4px 출처 (작업지시자 지시)

p71 항목별 페이지네이터 current_height vs 렌더러 실측 y 1:1 대조:

| pi | 페이지네이터 | 렌더러(상대) | 차이 |
|----|------|------|------|
| 348 | 130.0 | 130.0 | 0 |
| **349 (표)** | **195.2** | **212.1** | **+16.9** |
| 350~361 | — | — | ~23.6 고정 누적 |

발산은 **전적으로 TAC 표 pi=349** 에서 발생. 계측값:
- 페이지네이터 `typeset_block_table`: `table_total = effective_height(65.2)`
  (is_tac=attr&0x01=**false** → outer_margin 미포함, host_spacing=0).
- 렌더러 advance = **82.1 = 호스트 LINE_SEG(line_height+line_spacing) = fmt.total_height**.
- 즉 **treat_as_char 인라인 표를 페이지네이터는 "측정된 표 높이"로, 렌더러는 "호스트 한 줄"
  로 advance** → 16.9px 과소측정, 표 이후 누적 → 23.4px overflow.

### 실험 fix (treat_as_char 표 → fmt.total_height advance)

| | 페이지 | overflow | 노트 |
|--|------|------|------|
| Stage C baseline | 185 | 13 | 9쪽 |
| Stage D | 184 | 18 | 8쪽 |
| Stage D + TAC fix | 185 | **15** | 8쪽 |

- p71(23.4)·para642(19.7) **해결** ✓ (TAC 표 가설 정확).
- **그러나 신규**: para142 (Shape **40.6px** + 23.2px), para429(5.1) 발생.
- para142 는 빈 문단+Shape — TAC fix 가 앞선 표들에 높이를 더해 **Shape 를 페이지 밖으로
  밀어낸 연쇄 효과**(Shape advance 도 페이지네이터↔렌더러 불일치).

**확증**: 표 advance 만 고치면 불일치가 Shape advance 로 옮겨감(whack-a-mole, Stage 3-2
"전역 과보정" 재현). 노트 8쪽은 유지되나 overflow 는 깨끗이 안 줄어듦.

## 8. 결론·권고

- 단일 항목(표/Shape) 측정만 고치면 다른 항목으로 불일치가 이동 → **모든 per-item advance
  를 페이지네이터·렌더러가 공유**(B)해야 깨끗이 해결.
- (B) 규모: 렌더러 `layout_column_item` y-math(문단/TAC표/분할표/Shape advance)를
  height-only 함수로 추출 → 페이지네이터 동일 호출. Stage E~F·고위험·광범위 골든 재판정.
- **현 상태**: Stage D 스냅(+96, typeset.rs)만 작업트리 보존, 디버그/실험 코드 전부 제거.
  layout.rs 는 Stage C 커밋 상태. **커밋 보류 유지.**
