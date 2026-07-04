# Task 1293 Stage 26: 구분선 없음 미주 block flow 분석

## 목적

Stage25에서 큰 `미주 사이` 문서의 새 제목 tail 일부를 공통 로직으로 보정했지만,
`2024-11-practice-no-separator-above20-between20-below20`은 여전히 page 10 표시분에서
`문4` 제목/본문이 좌측 단 하단에 남고, PDF는 우측 단 상단으로 넘긴다. 이번 단계에서는
구분선이 없을 때 `구분선 위/아래`와 `미주 사이`가 separator line 없는 block에서 어떻게
소비되어야 하는지 분석한다.

## 우선 분석 대상

- target: `2024-11-practice-no-separator-above20-between20-below20`
- Stage25 final:
  - page count: 23/23/23
  - overflow: 37건
  - 첫 잔여 chain: page 9, `pi=464~466`

## 분석 계획

1. `compare_010.png`와 `dump-pages -p 9`를 기준으로 `문4`가 왜 좌측 단 하단에 남는지 확인한다.
2. 같은 원본의 `above0-between20-below2`와 비교해, separator line 유무/구분선 위아래 값이
   current height와 vpos base에 어떤 차이를 만드는지 확인한다.
3. `EndnoteSeparator len=0`일 때 separator item을 만들지 않는 현재 구조와,
   한컴이 보이는 미주 block 시작 간격이 서로 다른지 확인한다.
4. 수정이 필요하면 문서별 수치가 아니라 `separator_line_type == 0 && separator_length == 0`
   같은 공식 미주 모양 조건으로 제한한다.

## 확인 결과

### no-separator와 line separator 비교

`render_tree_010.json` 기준 같은 `pi=464~466`의 y 좌표가 크게 다르다.

| target | `pi=464` y | `pi=465` y | `pi=466` y | 판단 |
|---|---:|---:|---:|---|
| `above0-between20-below2` | 946.2 | 964.2 | 985.8 | frame 안쪽 |
| `no-separator-above20-between20-below20` | 1087.2 | 1105.3 | 1126.8 | frame 하단/밖 |

`dump-pages -p 9` 기준:

- `above0-between20-below2`
  - `EndnoteSeparator len=14173 above=0 below=566`
  - 단 0 `used=908.8px`, `hwp_used≈911.8px`, diff `-3.1px`
- `no-separator-above20-between20-below20`
  - `EndnoteSeparator len=0 above=5669 below=5669`
  - 단 0 `used=982.6px`, `hwp_used≈914.2px`, diff `+68.4px`

즉 구분선 없음 샘플은 invisible separator block이 current height와 render vpos 기준을 서로 다르게
만든다. 다만 단순히 `separator_line_type == 0 && separator_length == 0`일 때 새 제목 하단 허용을
막는 조건을 추가해도 overflow 수와 page/para grouping이 전혀 바뀌지 않았다.

## 폐기한 가설

- 가설: 구분선 없음 + 위/아래 margin이 있는 경우 새 번호 제목을 하단에 남기지 않으면 `pi=464~466`
  chain이 줄어들 것이다.
- 실험: `no_separator_margin_head_near_bottom` 조건을 추가해 `allow_compact_question_title_tail`과
  `large_between_notes_vpos_head_outside`에 반영했다.
- 결과:
  - `2024-11-practice-no-separator-above20-between20-below20`: overflow 37건 유지
  - `2024-11-practice-above0-between20-below2`: overflow 38건 유지
- 판단: 이 chain은 새 제목 advance 조건만으로 해결되지 않는다. separator block 이후의 render vpos
  base 또는 current height 누적 자체를 맞춰야 한다.
- 조치: 실익 없는 코드 변경은 되돌렸다. Stage26에는 분석 결과만 남긴다.

## 검증

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
- `cargo build --bin rhwp`: 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage26_no_separator_target --rhwp-bin target/debug/rhwp`
  - no-separator: page count 23/23/23, overflow 37건 유지
  - above0-between20-below2: page count 22/22/22, overflow 38건 유지

## 다음 단계

Stage27에서는 `EndnoteSeparator len=0`의 separator block이 `PageItem::EndnoteSeparator`와
렌더러 `HeightCursor`에서 어떤 y 기준으로 소비되는지 추적한다. 특히 `used`와 `hwp_used`가
서로 반대 방향으로 어긋나는 원인을 먼저 확인한다.
