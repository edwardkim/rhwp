# Task 1293 Stage 28: 잔여 TAC/shape 미주 overflow 분석

## 목적

Stage27에서 `구분선 없음 + 구분선위20 + 미주사이20 + 구분선아래20` 샘플의 첫 단
`문4` title-tail 분리는 해결했다. 이번 단계에서는 남은 renderer `LAYOUT_OVERFLOW` 중
가장 앞선 `pi=593`/`pi=613` 계열을 분석한다.

목표는 특정 문제 번호 위치를 수치로 밀지 않고, 미주 모양 설정과 TAC/shape 문단의 저장
`LINE_SEG`/render bbox가 pagination에서 어떻게 예약되어야 하는지 공통 로직으로 정리하는 것이다.

## 우선 대상

- `2024-11-practice-no-separator-above20-between20-below20`
  - Stage27: page count 23/23/23, overflow 4건
  - 잔여:
    - page 12, `pi=593`, Shape overflow 14.1px
    - page 13, `pi=613`, FullParagraph/Shape overflow 65~71px
- `2024-11-practice-above0-between20-below2`
  - Stage27: page count 22/22/22, overflow 38건
  - 보이는 구분선 + 미주사이20 계열의 후반 chain이 남아 있어 no-separator와 비교 대상으로 사용한다.

## 분석 계획

1. `dump-pages`와 `render_tree`에서 `pi=593`/`pi=613`의 문단 종류, TAC/shape bbox, 단 위치를 확인한다.
2. 같은 문단이 보이는 구분선 샘플과 구분선 없음 샘플에서 어떤 page/column에 놓이는지 비교한다.
3. overflow가 실제 content height 예약 부족인지, shape drawing bbox만 하단으로 나가는지 분리한다.
4. 수정이 필요하면 `para_has_tac_picture`, `shape` 예약 높이, 미주 전용 `HeightCursor` 경로 중
   공식 미주 모양과 연결되는 공통 조건에만 반영한다.

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo build --bin rhwp`
- target sweep:
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`

## 확인 결과

### Stage27 보정 유지 여부

Stage27의 핵심 수정은 보이는 구분선이 없는 큰 `미주 사이` 문서에서 pagination도 전체
`betweenNotes` 값을 번호 경계 gap으로 반영하는 것이다.

이번 단계에서 이를 되돌리고 `no_visible_separator_head_body_near_bottom` 조건만 추가하는 후보를
검토했다. 후보는 page 10의 `문4` 흐름은 유지했지만 전체 결과가 나빠졌다.

| target | Stage27 overflow | 후보 overflow | 판단 |
|---|---:|---:|---|
| `2024-11-practice-no-separator-above20-between20-below20` | 4 | 23 | 회귀 |
| `2024-11-practice-above0-between20-below2` | 38 | 38 | 변화 없음 |
| `2024-11-practice-above20-between0-below20` | 0 | 0 | 변화 없음 |

후보의 `compare_014.png`에서는 `pi=619` 근방 수식/텍스트 겹침 후보도 새로 생겼다. 따라서
Stage27의 전체 gap 반영은 되돌리지 않고 유지한다. 후보 코드는 폐기했고 현재 소스 diff는 없다.

### 남은 `pi=593`/`pi=613` 분석

Stage27 기준 남은 no-separator overflow는 다음 4건이다.

```text
LAYOUT_OVERFLOW: page=12, para=593, type=Shape, overflow=14.1px
LAYOUT_OVERFLOW_DRAW: section=0 pi=613 line=0 overflow=65.3px
LAYOUT_OVERFLOW: page=13, para=613, type=FullParagraph, overflow=65.3px
LAYOUT_OVERFLOW: page=13, para=613, type=Shape, overflow=71.4px
```

`dump-pages`와 render tree를 함께 보면 원인은 단순한 `미주 사이` gap 값이 아니다.

- page 13 표시분에서 `pi=593`은 `문19` 끝의 TAC 그림이다.
- page 14 표시분에서 `pi=613`은 `문21` 내부의 TAC 그림이다.
- 둘 다 빈 문단 + TAC 그림 문단이며, 저장 vpos와 renderer `HeightCursor`가 실제 drawing y를
  pagination의 sequential height보다 아래로 유지한다.
- 특히 page 14는 PDF가 `문21` 그래프 묶음을 왼쪽 단 상단부터 보여 주는데, rhwp는 `문20` 본문이
  page 14 왼쪽 단 앞부분을 차지한다. 즉 `pi=613` 자체만 다음 단으로 미는 수정은 근본 해결이
  아니다. 앞쪽 `문15~문20` 흐름에서 TAC/shape 묶음 높이 예약과 vpos rewind 처리가 누적되어
  한컴/PDF보다 늦어진다.

## 결론

Stage28에서는 코드 수정을 채택하지 않는다.

- Stage27의 no-separator 전체 `betweenNotes` gap 보정은 page 10의 한컴/PDF 흐름을 맞추는 데
  필요하므로 유지한다.
- 남은 문제는 `betweenNotes` 전역 gap이 아니라 compact 미주 내부의 TAC 그림 문단이 renderer
  y에서는 하단으로 내려가고 pagination은 그 차이를 충분히 반영하지 못하는 문제다.
- 다음 스테이지에서는 `HeightCursor`와 pagination이 TAC 그림 문단의 실제 drawing bottom을 같은
  기준으로 쓰도록 좁혀 수정한다.

## 실행한 검증

- `cargo build --bin rhwp`: 통과
- `dump-pages`:
  - `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp -p 12`
  - `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp -p 13`
- `RHWP_VPOS_DEBUG=1 export-render-tree`:
  - page 12: `pi=593` TAC 그림과 다음 `pi=594` 제목 흐름 확인
  - page 13: `pi=613` TAC 그림과 다음 `pi=614` 단 전환 확인

## 다음 단계

Stage29에서는 `pi=593`/`pi=613`을 문단 번호로 특별 처리하지 않고, 다음 공통 조건을 기준으로
수정 후보를 만든다.

- compact 미주 흐름
- TAC 그림/도형 only 문단
- 저장 vpos rewind 또는 lazy/page base와 실제 drawing bottom이 어긋나는 경우
- 다음 미주 제목 또는 다음 본문이 같은 단 하단에서 overflow/overlap 후보를 만드는 경우
