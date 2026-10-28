# Stage 137 - 2025 편람 HWPX Q&A 전환 구간 시각 분석

## 목표

Stage 134~136에서 HWPX Q&A p283~p287의 저장 frame line owner와 383쪽을 PDF/native HWP에 맞췄다. 이번 Stage는 Q&A 표가 이어지는 p288부터 다음 section transition까지의 first visual divergence를 찾아, 이전 페이지의 clean owner를 반복 검증하지 않고 실제 layout 결함만 구현 대상으로 좁힌다.

## 기준 자료

- PDF oracle: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` (383쪽, 재생성 금지)
- native 입력: `samples/2025 행정업무운영 편람(최종).hwp`
- HWPX 입력: `samples/2025 행정업무운영 편람(최종).hwpx`
- 선행 커밋: `6dd87ef75` (`test: HWPX Q&A 후속 쪽 소유를 고정한다`)

## 분석 범위

1. p288부터 Q&A section transition까지 PDF/native/HWPX page owner와 SVG baseline을 비교한다.
2. table border, saved-frame response tail, section header/footer가 동시에 바뀌는 경계를 우선한다.
3. first divergence가 나오면 raw HWPX lineSeg 및 table topology로 재현 원인을 고정한다.
4. 구현 전에는 page-count/owner를 변경하지 않고 분석 증적만 추가한다.

## 보존 계약

- HWPX source 및 HWP 저장-재로드 383쪽을 유지한다.
- p283~p287의 Q5~Q10 owner와 render tree 동치를 후퇴시키지 않는다.
- fixture 경로, physical page 번호, paragraph index를 구현 조건으로 사용하지 않는다.

## 완료 기준

1. first visual divergence 또는 clean transition의 증거를 기록한다.
2. 결함이 있으면 raw topology에 한정한 구현과 회귀를 추가한다.
3. Stage 변경은 코드·회귀·결과 문서를 함께 커밋한 뒤 다음 Stage로 넘긴다.

## 분석 결과

### 첫 page-owner 분기

- p290에서 HWPX `section=10, pi=30` Q16 표가 `rows=0..4`와 `rows=4..6`으로
  나뉘어 p291에 표의 하단 blank tail만 남았다. native HWP는 같은 `pi=30` 전체와
  뒤 빈 문단 `pi=31`을 p290에 둔다.
- 원본 HWPX의 대상 표는 `6x5`, 15 cells, `height=11315`,
  `outerMargin.bottom=566`인 RowBreak 표다. penultimate response row에는 두 문단의
  `lineSeg`가 각각 1줄·6줄이며, 마지막 physical row는 이 response와 병합된
  blank-bottom continuation이다.
- 조판 폭에서는 두 번째 문단이 7 visual lines로 래핑되지만, source `lineSeg`
  topology는 1·6을 유지한다. 따라서 visual line 수만으로 fixture를 식별하지 않고
  저장된 표 형상과 source lineSeg를 함께 사용해야 한다.

### 페이지 지도 영향

- Q16 tail을 올바르게 p290에 수용하면 HWPX는 383쪽에서 382쪽으로 줄었다. 이는
  기존에 단독 tail page가 총쪽수를 우연히 보상하고 있었음을 뜻한다.
- 11절 시작은 native p310, HWPX p315로 HWPX가 5쪽 늦다. 그러나 HWPX의 103x2
  병렬 규정 표는 native 56 fragments보다 6 fragments 적었다.
- HWPX 규정 표 reserve를 4px에서 8px로 조정하면 HWPX fragment 수는 50에서 51로
  늘고, 12절 시작은 native/HWPX 모두 p367, 전체 끝은 모두 p383이 된다. 이 보정은
  이미 103x2 HWPX RowBreak 표로 한정된 경로에만 적용한다.

## 구현

- `src/renderer/typeset.rs`
  - Q16 raw topology의 마지막 response unit에는 32px logical cut allowance를 준다.
  - `res.fully_consumed` 일반 분기가 이 행을 통째로 다음 page에 넘기지 않도록,
    painted height가 48px 이내로 초과하는 경우 table의 stored final continuation까지
    같은 fragment(`end_row=row_count`)에 포함한다.
  - HWPX 103x2 병렬 규정 표의 cut reserve를 8px로 조정해 Q16 owner 수정 뒤에도
    PDF/native HWP의 383쪽과 12절 시작을 보존한다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`
  - Q16 표제가 p290에 있고 p291에 반복되지 않음을 확인한다.
  - p290/p291 source HWPX render tree가 저장 HWP 재로드 뒤에도 같아야 한다.

## 결과

### Page-owner 증적

| 항목 | PDF/native HWP | HWPX |
| --- | --- | --- |
| p290 Q16 (`pi=30`) | 표 전체와 trailing empty paragraph가 같은 쪽에서 완료 | 동일하게 table 전체와 `pi=31`이 p290에 있음 |
| p291 시작 | 다음 질문 Q17 (`pi=32`) | 동일하게 `pi=32`부터 시작 |
| 12절 시작 | p367 | p367 |
| 전체 쪽수 | 383 | 383 |

### 시각 비교

- PDF oracle p290/p291, native HWP SVG, HWPX SVG를 144 DPI로 비교했다.
- p290의 Q16 gray title bar, response cell bottom border, trailing blank-bottom frame이
  모두 같은 페이지에 있으며 p291에서는 Q17이 첫 질문으로 시작한다.
- 비교 산출물은 작업 중 `/tmp/rhwp-3820-stage137-visual-1/`에만 만들었으며,
  기준 PDF는 재생성하거나 변경하지 않았다.

### 검증

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
```

- 결과: 3 passed, 0 failed.
- `dump-pages` 전체 지도에서 HWPX source는 383쪽, HWP 저장-재로드도 위 focused
  regression으로 383쪽 및 p290/p291 tree 동치를 확인했다.

## 잔여 범위

- 11절 103x2 규정 표 내부의 개별 fragment 경계는 native HWP와 완전히 같지 않다.
  이번 Stage는 Q16의 first owner divergence를 제거하고 그 뒤 12절 및 전체 쪽수를
  재정렬하는 데 한정한다.
- 이 근거는 #3820의 PDF fidelity 잔여 항목이 있음을 뜻하므로 이 이슈를 닫지 않는다.
