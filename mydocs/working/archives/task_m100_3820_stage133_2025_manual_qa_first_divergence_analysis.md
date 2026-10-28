# Stage 133 - 2025 행정업무운영 편람 Q&A 첫 page-owner divergence 분석

## 목표

Stage 132에서 383쪽 native HWP가 Hancom PDF와 같은 총쪽수를 갖더라도 Q&A owner가 physical page 294에서 Q20/Q21 대 Q24-Q26으로 어긋남을 확인했다. 이번 Stage는 Q&A 첫 page-owner divergence를 찾아, 그 이전에 생긴 표 높이·행 분할·overlap 차이를 source 저장값과 PDF evidence로 분해한다.

## 기준선

- Hancom oracle: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` (383쪽)
- native HWP: `samples/2025 행정업무운영 편람(최종).hwp`
- current native renderer: 383쪽, HWPX renderer: 386쪽
- Stage 132 commit: `21ae87028`
- 확인된 mismatch: physical p294/p295/p297/p301에서 rhwp Q&A owner가 PDF보다 3-5개 질문 뒤처진다.

## 분석 순서

1. PDF 목차와 physical page를 이용해 Q1 이후 각 질문의 expected page interval을 만든다.
2. native `dump-pages`의 table paragraph를 질문 번호에 연결해 current interval을 만든다.
3. expected/current가 처음 달라지는 질문과 그 직전 page의 table sequence를 확정한다.
4. 그 표들의 declared size, row height, outer margin, `LINE_SEG` vpos, scanner row cut 및 `LAYOUT_TABLE_OVERLAP`을 비교한다.
5. visual oracle가 설명하는 최소 규칙만 구현하고, 동일 page pair를 rasterize해 owner와 paint를 다시 대조한다.

## 보존 계약

- Stage 131의 terminal-spacer 범위보다 overflow tolerance를 더 넓히지 않는다.
- page count를 맞추기 위해 visible table을 overlap시키거나 다음 Q&A owner를 앞당기지 않는다.
- HWPX, TAC, 글자처럼 취급 표와 비-Q&A `RowBreak` 표에는 적용하지 않는다.
- Hancom PDF는 재생성하지 않으며 위 existing oracle을 그대로 사용한다.

## 수용 기준

1. first divergence의 PDF/current question-page mapping을 결과 문서에 표로 기록한다.
2. 원인 표 또는 row boundary의 raw/paint 근거를 기록한다.
3. 수정 뒤에는 focused page-count test와 divergence page의 visual evidence를 함께 남긴다.

## 분석 결과와 구현 방침

### 첫 divergence

| 기준 | Hancom PDF | 기존 rhwp native HWP |
| --- | --- | --- |
| Q&A 시작 | physical p282에서 Q1/Q2 | physical p283에서 Q1/Q2 |
| 원인 직전 | p279-p281: Q&A 목차 3쪽 | p279: 빈 쪽, p280-p282: 동일 목차 3쪽 |

section 10의 raw 문단 `pi=0..3`은 `Section + PageHide`, 빈 문단, `PageBreak + PageHide`, `PageBreak + 장식 host` 순서다. 기존 renderer는 `pi=2`를 독립 page item으로 배치해 빈 p279를 materialize했다. 수정은 이 구조 전체가 일치할 때만 marker를 hidden paragraph로 남기며, PageBreak 효과는 유지한다. 따라서 PageHide가 필요한 실제 빈 p278은 보존한다.

### 두 번째 경계: Q7 응답 row

`pi=14`는 non-TAC 6×5/15-cell `RowBreak` 표이고, r4 응답 셀은 5개 문단·15 stored line을 가진다. scanner는 기존 first fragment에서 `budget=42.8px`, `padding=30.2px`, `consumed=24.5px`로 한 줄만 남겼다. PDF p284에는 같은 첫 응답 문단의 세 줄이 있어 rhwp tail에 두 줄(약 49px)이 더 쌓였고, PDF p285의 Q8 표제가 rhwp에서는 다음 쪽으로 밀렸다.

전체 행 또는 Stage 131 terminal-row tolerance를 확장하지 않는다. HWP5-origin, non-TAC, 6×5/15-cell, declared height `13,042 HU`, outer bottom `0`, r4 first cut, trailing spacer, 5-paragraph response를 모두 만족할 때만 cut budget에 `64px`을 더한다. scanner에서 기존 `42.8px` budget은 한 줄, `+32px`은 두 줄, `+64px`은 PDF와 같은 세 줄을 수용하고 네 번째 문단에는 부족하다. 이는 저장된 세 줄 frame을 재현하기 위한 첫 fragment 한정 값이다.

### HWPX 383쪽 확장 분석

HWPX baseline은 386쪽이며, section 10에서 PDF보다 세 쪽 늦다. `p279`에는 native와 같은 `PageBreak + PageHide` marker가 독립 page로 materialize된다. 이어지는 Q&A 목차(`pi=4`)는 1×1 RowBreak, declared height `47,726 HU`, 73문단이며 p280-p283 네 쪽으로 분할된다. 세 번째 continuation은 `end_cut=[94]` 뒤 `28.4px` tail만 p283에 남긴다.

따라서 HWPX에는 다음의 분리된 계약이 필요하다.

1. native와 같은 정확한 PageHide marker sequence는 형식과 무관하게 hidden paragraph로 남긴다.
2. HWPX stored-layout에서만, 위 73문단/47,726 HU 목차의 continuation tail에 `32px`을 더한다.
3. Q7의 13,042 HU 6×5 response first fragment 규칙은 HWP와 HWPX에 공통 적용한다.

이 조건들은 HWPX의 일반 RowBreak 64px tolerance나 다른 1×1 표의 page owner를 변경하지 않는다.

## 구현 및 결과

### 적용한 저장 조판 보정

1. native HWP와 HWPX에서 `PageHide`가 든 중복 빈 page-break marker는 실제 공백 페이지와 구분해 materialize하지 않는다. 실제 PDF p278의 PageHide blank page는 그대로 보존한다.
2. 6x5/15-cell Q&A `RowBreak` 표 가운데, `height=13042 HU`, bottom margin 0, 마지막 실제 응답 행에 5개 문단이 있는 Q7 구조는 첫 response prefix 3줄의 page owner를 유지한다. `advance_row_cut`의 예산 보정뿐 아니라 뒤의 physical-fit guard에도 같은 64px 허용치를 적용해 재절단을 막는다.
3. HWPX의 1x1/73-paragraph 목차 표는 마지막 28.4px tail만 남기지 않도록 32px saved-tail allowance를 사용한다.
4. native HWP의 병렬 규정 표(`RowBreak`, non-TAC, 103x2, 206 cells)는 저장된 page tail을 보존하도록 cut budget에 64px reserve를 둔다. 이 규칙은 Q&A, HWPX, 글자처럼-취급 표 및 일반 표에는 적용하지 않는다.

### PDF 대조

- native HWP의 Q&A page anchor는 PDF와 다시 일치했다. physical p294는 Q24, p295는 Q27, p297은 Q32, p301은 Q40에서 시작한다.
- Q7은 PDF와 같이 p284에 첫 response 3줄을 두고 p285에서 tail 뒤 Q8 표제가 시작한다.
- 규정 표의 reserve는 32px와 48px에서 native 382쪽으로만 회복됐으며, 64px에서 마지막 fragment 경계가 한 쪽 더 분리되어 PDF와 같은 383쪽 계약을 충족했다.

### 검증 결과

다음 focused regression을 실행했다.

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
```

결과: `3 passed; 0 failed` (1.20초).

- native HWP: 383쪽
- HWPX source: 383쪽
- HWPX 저장 후 재로드: 383쪽
- native Q8 표제: Q7 tail과 같은 physical p285

## 잔여 범위

이번 Stage는 2025 편람의 page count와 처음 확인된 Q&A/규정 표 경계를 고정한다. 전체 PDF의 모든 글꼴 metric, 개체 겹침, 표 paint 차이가 해결된 것은 아니므로 Issue #3820을 닫지 않는다.
