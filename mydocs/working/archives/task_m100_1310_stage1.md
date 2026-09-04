# Task #1310 Stage 1 - TAC 수식-only 자동 줄바꿈 진단

## 1. 범위

- 이슈: #1310 `수식-only 흐름의 자동 줄바꿈 조판 구현`
- 샘플: `samples/3-09월_교육_통합_2022.hwp`
- 기준 페이지: 10쪽, 오른쪽 단 미주 수식 블록
- 핵심 질문: 한 줄에 TAC 수식이 연달아 있을 때, 다음 TAC 수식까지 현재 줄에 배치 가능한지 폭 기준으로 판단하는가?

## 2. 관찰 증거

기준 출력:

- `output/poc/ci27052730125-fixed/3-09월_교육_통합_2022_010.svg`
- `output/poc/ci27052730125-fixed-tree/render_tree_010.json`

`dump-pages` 기준으로 10쪽 오른쪽 단의 본문 영역은 다음과 같다.

| 항목 | 값 |
|---|---:|
| body x | 34.0 |
| body y | 90.7 |
| body w | 725.7 |
| body h | 1001.6 |
| column 1 x | 402.5 |
| column 1 w | 357.2 |
| column 1 right | 759.7 |

문제 후보는 렌더러가 구성한 가상 미주 문단이다.

```text
FullParagraph[미주] pi=574 vpos=238410..246663 "(빈)"
```

`pi=574`는 원본 섹션 문단 번호가 아니라 미주 렌더링 과정에서 만들어진 가상 문단이다. 실제 `section 0` 문단 수는 468개라서 `rhwp dump -s 0 -p 574`로는 접근되지 않는다.

해당 문단의 TextLine/Equation 배치는 다음과 같다.

| line y | line w | TAC 수식 폭 | TAC 총폭 | 마지막 right | 판정 |
|---:|---:|---|---:|---:|---|
| 555.7 | 357.2 | - | 0.0 | 0.0 | 선행 guide/빈 줄 |
| 593.1 | 357.2 | 78.2 + 102.2 + 182.8 | 363.2 | 765.8 | column right 759.7을 약 6.1px 초과 |
| 630.4 | 357.2 | 112.2 + 117.9 | 230.1 | 632.7 | 폭 안에 들어감 |
| 665.8 | 357.2 | 102.2 + 41.9 + 17.5 | 161.6 | 564.1 | 폭 안에 들어감 |

첫 실제 수식 줄은 줄 폭 `357.2px`보다 TAC 수식 총폭 `363.2px`가 크다. 따라서 한컴 기준처럼 수식 단위 자동 줄바꿈을 하려면, 세 번째 TAC 수식을 같은 줄에 붙이기 전에 줄넘김 판단이 필요하다.

## 3. 현재 코드 경로

### 3.1 TAC 수식 줄 배정

`src/renderer/layout/paragraph_layout.rs`의 `equation_only_tac_line_assignment()`는 TAC 수식을 기존 `ComposedLine`에 배정한다.

핵심 동작:

- 모든 줄의 `runs`가 비어 있는 수식-only 문단만 대상으로 한다.
- `char_start`가 연속 동일/감소하는 degenerate line-seg 케이스에서만 동작한다.
- 같은 position의 TAC 묶음을 같은 position의 기존 줄 후보에 순서대로 배분한다.
- TAC 수가 줄 수보다 많으면 남는 TAC는 마지막 줄에 모은다.

즉 이 함수는 "기존 줄 후보에 어느 TAC를 놓을지"만 결정한다. "다음 TAC를 같은 줄에 놓으면 사용 가능 폭을 넘는지"는 판단하지 않는다.

### 3.2 실제 렌더링

`src/renderer/layout/paragraph_layout.rs`의 empty-run TAC-only 렌더링 경로는 다음 순서로 동작한다.

1. `tac_offsets_px`에서 TAC 수식 폭을 모은다.
2. `equation_only_tac_line_assignment()` 결과로 현재 줄에 속한 TAC만 필터링한다.
3. 현재 줄에 속한 TAC 폭을 합산해 셀 내부 정렬에만 사용한다.
4. `inline_x = tac_base_x + align_offset`에서 시작한다.
5. 현재 줄에 속한 TAC를 순서대로 그리고 `inline_x += tac_w`로 오른쪽에 이어 붙인다.

이 루프에도 `inline_x + tac_w > line_right`와 같은 자동 줄넘김 판단이 없다.

### 3.3 Composer

`src/renderer/composer.rs`의 `compose_lines()`는 HWP `LINE_SEG`와 명시적 강제 줄바꿈을 기준으로 `ComposedLine`을 만든다.

현재 동작:

- `line_segs`가 있으면 저장된 line-seg 범위를 보존한다.
- `\n`은 별도 visual line으로 분할한다.
- line-seg가 없는 fallback에서만 문자 수 휴리스틱으로 텍스트 줄바꿈을 한다.
- TAC 수식-only 문단의 폭 기반 자동 줄바꿈은 수행하지 않는다.

따라서 현재 rhwp는 TAC-only 수식 흐름에서 자동 줄바꿈을 composer 단계에서도 하지 않고, layout 단계에서도 하지 않는다.

## 4. 결론

사용자가 지적한 핵심은 확인되었다.

현재 rhwp는 한 줄에 TAC 수식이 연달아 있을 때 다음 TAC까지 현재 줄에 배치 가능한지 판단하지 않는다. 저장된 `LINE_SEG`/기존 `ComposedLine` 슬롯에 TAC를 배정한 뒤, 같은 줄로 판정된 TAC 수식들을 폭 초과 여부와 무관하게 이어 붙인다.

이 때문에 한컴은 수식-only 흐름을 여러 줄로 자동 배치하는 반면, rhwp는 첫 수식 줄에서 column right를 넘는 조판이 발생한다.

## 5. 구현 가설

아래 내용은 구현 결론이 아니라, Stage 2에서 검토할 수 있는 작업 가설이다. 최종 처리 방향은 작업지시자의 판단에 따른다.

가설 A - layout 단계의 TAC-only packing 문제:

- `equation_only_tac_line_assignment()`를 단순 line assignment에서 폭 기반 line packing을 반환하는 공통 helper로 확장할 수 있다.
- 입력은 TAC 순서, TAC 폭, 기존 `ComposedLine` 메트릭, available width, 선행 guide line 여부로 제한한다.
- 같은 줄에 다음 TAC를 추가했을 때 `current_width + next_tac_width > available_width`이면 새 virtual visual line으로 넘긴다.
- `[수식]` 표시는 debug/control-code overlay 성격이므로 실제 본문 문자처럼 폭 계산 단위에 섞지 않는다.
- 렌더링과 커서 이동이 같은 packing 결과를 보도록 공통 구조를 둔다.
- #1308에서 처리한 강제 줄바꿈 이후 내어쓰기/커서 이동 회귀를 깨지 않도록 `eq-002`와 CI 회귀 샘플을 함께 검증한다.

가설 B - composer/IR line model 문제:

- 한컴이 수식-only 흐름을 단순 렌더링 단계가 아니라 line-seg 해석 또는 내부 line model 단계에서 이미 재분배하고 있을 수 있다.
- 이 경우 layout 단계에서만 virtual line을 만들면 SVG는 맞출 수 있어도 커서 이동, 선택 영역, hit-test, 페이지네이션이 같은 줄 구조를 공유하지 못할 수 있다.
- 이 가설이 맞다면 `ComposedLine` 또는 그와 동등한 중간 구조에서 TAC-only visual line을 표현하는 쪽을 검토해야 한다.

가설 C - TAC 폭/단위 계산 문제:

- 현재 관찰된 overflow가 실제 wrap 부재 때문이 아니라 수식 폭, control-code 표시 폭, 여백, 단 폭 계산 중 하나가 과대/과소 계산된 결과일 수 있다.
- 이 경우 packing을 추가해도 줄넘김 위치가 한컴 기준과 다르게 나올 수 있다.
- 먼저 Equation bbox, TAC common.width, line segment width, column width가 같은 좌표계에서 비교되는지 확인해야 한다.

가설 D - text/TAC/fixed-tab 혼합 순서 문제:

- 이번 샘플의 문제는 수식-only처럼 보이지만, 실제 내부 흐름에는 TAC 수식, 일반 글자, 고정 탭, control marker가 섞여 있을 수 있다.
- 이 경우 수식 TAC만 묶어 packing하면 특정 샘플은 맞아도 text+TAC 혼합 문단에서 순서 또는 커서 위치가 다시 틀어질 수 있다.
- 이 가설이 맞다면 TAC-only 전용 처리보다 inline item sequence를 먼저 정확히 복원하는 작업이 우선일 수 있다.

## 5.1 가설 A가 성립하지 않을 때 직면하는 상황

가설 A가 실패하면 문제는 단순히 "연속 TAC 폭을 합산해서 줄을 넘기는 처리"가 아니라는 뜻이다. 그 경우 다음 상황에 직면한다.

- 렌더링만 맞고 커서 이동/선택/hit-test가 틀어지는 상황: virtual line이 renderer 안에만 존재하고 editor model에는 없기 때문이다.
- 특정 페이지는 맞지만 페이지네이션이 흔들리는 상황: 새로 생긴 visual line 높이가 다음 미주/본문 흐름을 밀어야 하는데 pagination 단계가 그 변화를 모를 수 있다.
- HWP와 HWPX 또는 SVG와 웹 canvas가 서로 다르게 맞는 상황: 폭 계산 또는 line model 공유 지점이 포맷별/렌더러별로 다를 수 있다.
- #1308에서 고친 강제 줄넘김/내어쓰기/커서 이동이 회귀하는 상황: 같은 TAC-only 문단이라도 강제 줄넘김 기반 흐름과 자동 줄넘김 기반 흐름의 시작 x 규칙이 다를 수 있다.
- 더 큰 범위의 inline layout 재정의가 필요한 상황: TAC 수식, 일반 글자, 고정 탭, control marker를 하나의 순서 있는 inline item 흐름으로 보고 줄을 구성해야 할 수 있다.

## 6. Stage 2 검증 후보

- `samples/3-09월_교육_통합_2022.hwp` 10쪽 SVG/render-tree
- `samples/eq-002.hwp`
- `samples/hwpx/eq-002.hwpx`
- `tests/issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`
- #1308 강제 줄넘김/커서 이동 관련 테스트

## 7. 다음 승인 요청

Stage 2에서는 위 관찰과 가설을 바탕으로 구현 선택지를 더 좁혀 제시한다. 적용 여부와 최종 처리 방식은 작업지시자의 승인 후 진행한다.
