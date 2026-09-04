# Task 1293 Stage 30: 구분선 없음 TAC 그림 잔여 overflow 분석

## 목적

Stage29에서 보이는 구분선 + 큰 `미주 사이` 문서의 local rewind fit 보정은 채택했다.
그러나 `구분선 없음 + 구분선위20 + 미주사이20 + 구분선아래20` target의 잔여 overflow 4건은
그 보정을 적용하면 page count가 `23/23/23`에서 `24/24/23`으로 회귀한다.

이번 단계에서는 구분선 없음 문서를 별도 수치로 밀지 않고, Stage27의 전체 `미주 사이` gap 소비와
TAC 그림 문단의 renderer y/pagination y가 함께 맞아야 하는 공통 조건을 다시 좁힌다.

## 우선 대상

- target: `2024-11-practice-no-separator-above20-between20-below20`
- 샘플: `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
- Stage29 결과:
  - page count: `23/23/23`
  - overflow:
    - page 12, `pi=593`, Shape overflow 14.1px
    - page 13, `pi=613`, FullParagraph/Shape overflow 65~71px

## 현재 관찰

`dump-pages -p 12/-p 13` 기준:

- page 13 표시분 우측 단 마지막에 `pi=593` TAC 그림이 있고, 다음 `pi=594`는 `문20` 제목이다.
- page 14 표시분 좌측 단에는 `문21` 내부 TAC 그림 문단 `pi=607`, `pi=609`, `pi=611`, `pi=613`이
  반복 배치된다.
- `pi=607`은 저장 vpos가 같은 미주 제목(`pi=603`) 쪽으로 되감기는 그림 문단이고, renderer는 이전
  content floor를 침범하는 되감김을 순차 y로 유지한다.
- 보이는 구분선 문서와 달리 같은 crossing 보정을 그대로 적용하면 no-separator page count가 24쪽으로
  늘어난다. 따라서 잔여 문제는 "TAC 그림만 보수적으로 넘기기"가 아니라 구분선 없음 block의 gap 소비와
  그림 묶음 높이 예약의 균형 문제로 본다.

## 분석 계획

- `RHWP_DEBUG_TAC_CURSOR=1 RHWP_VPOS_DEBUG=1 export-render-tree`로 `pi=592~594`와 `pi=607~614`의
  renderer y 입력/출력, vpos rewind skip 여부를 확인한다.
- `dump-pages`와 비교해 pagination이 어느 문단에서 renderer보다 덜/더 예약하는지 분리한다.
- 수정 후보가 필요하면 다음 조건을 만족해야 한다.
  - page count `23/23/23` 유지
  - no-separator overflow 4건 감소
  - visible `above0-between20-below2`의 Stage29 개선 유지
  - `above20-between0-below20` 0 overflow 유지

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo build --bin rhwp`
- target sweep:
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`

## 확인 결과

### Stage24 검토

Stage24는 준비 단계가 아니라 이미 커밋된 전체 sweep 기준점이다.

- 커밋: `f4e662ca task 1293: 미주 설정 전체 sweep 결과 기록`
- 결과 문서: `mydocs/working/task_m100_1293_stage24.md`
- 실행 산출물: `output/task1293_stage24_full_sweep/summary.json`
- 전체 15개 target의 SVG/PDF/render tree page count는 모두 1:1이었다.
- 다만 `overflow_lines`가 남아 있어 `task_m100_1293_impl.md`의 "미주 기능 완료" 목적에는 아직
  도달하지 못했다.

### 폐기한 가설 1: no-separator short tail 허용

임시로 `구분선 없음 + 큰 미주 사이 + 직전 TAC 그림` 조건에서 하단 short tail을 현재 단에 남기도록
`advance_for_fit` 예외를 추가했다.

- 산출물: `output/task1293_stage30_narrow_tail_trial/summary.json`
- 결과:
  - 기존 Stage29: overflow 4건
  - 후보: overflow 7건
  - 새 overflow:
    - page 10, `pi=537`, Shape overflow 10.2px
    - page 10, `pi=538`, FullParagraph overflow 46.1px
- 판단:
  - `pi=538`을 억지로 같은 단에 남기면 직전 `pi=537` TAC 그림의 실제 renderer y와 겹쳐 frame을
    더 침범한다.
  - short tail 허용은 overflow를 줄이지 못하고 새 overflow를 만든다.
- 조치:
  - 임시 조건과 `RHWP_DEBUG_ENDNOTE_FLOW` 로그는 제거했다.

### 폐기한 가설 2: no-separator에서 전체 미주 사이 gap 대신 초과분만 반영

Stage27의 핵심 수정은 보이는 구분선이 없는 큰 `미주 사이` 문서에서 pagination도 전체
`betweenNotes` 값을 번호 경계 gap으로 반영하는 것이다. 이 조건이 과도한지 확인하기 위해
일시적으로 no-separator도 기존 초과분 방식으로 되돌렸다.

- 산출물: `output/task1293_stage30_no_separator_extra_gap_trial/summary.json`
- 결과:
  - 기존 Stage29: overflow 4건
  - 후보: overflow 37건
  - page 10의 `pi=464~466` 문4 chain이 다시 overflow로 돌아왔다.
- 판단:
  - Stage27의 전체 gap 반영은 no-separator 첫 단 하단 under-count를 막기 위해 필요하다.
  - 이를 되돌리면 page13/14 일부 drift가 예전처럼 보이더라도 핵심 overflow가 대량 회귀한다.
- 조치:
  - 임시 변경은 제거하고 Stage29 기준 코드를 유지했다.

### 남은 원인 분리

Stage29 기준 no-separator 잔여 overflow는 4건이다.

```text
LAYOUT_OVERFLOW: page=12, para=593, type=Shape, overflow=14.1px
LAYOUT_OVERFLOW_DRAW: section=0 pi=613 line=0 overflow=65.3px
LAYOUT_OVERFLOW: page=13, para=613, type=FullParagraph, overflow=65.3px
LAYOUT_OVERFLOW: page=13, para=613, type=Shape, overflow=71.4px
```

`render_tree`와 `RHWP_DEBUG_TAC_CURSOR=1 RHWP_VPOS_DEBUG=1 export-render-tree` 기준:

- `pi=593`
  - render tree 실제 그림 bbox는 `y=833.2`, `h=197.6`으로 body frame 안쪽이다.
  - 그러나 뒤따르는 `pi=594` 문20 제목은 `y=1106.4`에 놓이며 하단 bleed tolerance로만 통과한다.
  - 단순 shape overflow 오탐처럼 보이지만, 다음 제목 흐름까지 보면 여전히 단 하단 tail 문제다.
- `pi=613`
  - renderer 로그: `TAC_CURSOR FullPara pi=613 y_in=1038.4 y_out=1163.6`
  - render tree bbox도 `y=1038.4`, `h=119.2`로 실제 frame 밖이다.
  - `HeightCursor`는 저장 vpos 기준으로도 더 위로 당길 근거가 없어 `result=1038.4`를 유지한다.

따라서 `pi=593`과 `pi=613`은 같은 숫자 보정으로 처리하면 안 된다.

- `pi=593`: cursor/후행 title tail 판정과 실제 bbox 판정의 차이를 분리해야 한다.
- `pi=613`: 실제 TAC 그림 bbox가 frame 밖으로 나가므로 pagination이 해당 그림 묶음의 drawing
  bottom을 더 정확히 예약해야 한다.

## 결론

Stage30에서는 코드 수정을 채택하지 않는다.

- Stage27의 no-separator 전체 `betweenNotes` gap 반영은 유지한다.
- short tail을 억지로 현재 단에 남기는 방법은 새 overflow를 만든다.
- 전체 gap 반영을 되돌리는 방법은 page10 overflow chain을 대량 회귀시킨다.
- 다음 스테이지에서는 단순 하단 tail 분기가 아니라, render tree bbox와 pagination의 current height가
  함께 보는 "실제 content bottom" 기준을 만들고, `pi=593`처럼 cursor만 넘는 경우와 `pi=613`처럼
  drawing bbox가 넘는 경우를 나누어 처리한다.
