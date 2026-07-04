# Task 1293 Stage 9: between=0 및 구분선 없음 잔여 flow 분석

## 목적

Stage8-A에서 `미주 사이 20mm` 초과분 전체 예약으로
`2024-11-practice-above0-between20-below2`는 PDF와 22/22로 맞았다. 남은 mismatch는 다음 세
가지다.

| target | PDF | rhwp | 우선 의심 |
|---|---:|---:|---|
| `2024-11-practice-shape987` | 21 | 22 | 9/8/7 샘플 전용 후반 flow 지연 |
| `2024-11-practice-above20-between0-below20` | 21 | 20 | `between=0`에서 문단 간 vpos 되감기를 같은 단에 겹쳐 붙임 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 21 | 구분선 없음 20/20/20의 후반 flow 과소 예약 |

이번 스테이지는 먼저 `between=0` 샘플의 시각 overlap과 page-count 과소 문제를 해결한다.
`between=0`은 공식 설정상 번호 사이 간격은 0mm지만, 저장 LINE_SEG/vpos가 단 경계를 암시하면
같은 단에 겹쳐 렌더하면 안 된다.

## 초기 단서

`2024-11-practice-above20-between0-below20` p10 dump:

- `pi=471` 문6 제목의 `vpos=60460`
- `pi=474` 이후 `pi=475`가 다시 `vpos=60460`으로 되감긴다.
- 현재 로직은 단 하단 85% 이상일 때만 문단 간 vpos rewind를 단 이월로 처리한다.
- 이 케이스는 단 초반/중반에서도 이전 paragraph 영역을 침범할 수 있어 시각 overlap과 page-count
  과소가 함께 생긴 것으로 보인다.

## 추가 샘플 확인

작업지시자가 요청한 2024년 11월 실전 미주 모양 분리 샘플은 다음 8종으로 확인했다. 각 샘플은
`samples/`에 `hwp/hwpx`가 있고, `pdf/`에 같은 이름의 한컴 기준 PDF가 있다.

| sweep target | 샘플 설정 |
|---|---|
| `2024-11-practice-shape987` | 구분선 위 9mm, 미주 사이 8mm, 구분선 아래 7mm |
| `2024-11-practice-above0-between0-below0` | 구분선 위 0mm, 미주 사이 0mm, 구분선 아래 0mm |
| `2024-11-practice-above0-between7-below2` | 구분선 위 0mm, 미주 사이 7mm, 구분선 아래 2mm |
| `2024-11-practice-above0-between7-below20` | 구분선 위 0mm, 미주 사이 7mm, 구분선 아래 20mm |
| `2024-11-practice-above0-between20-below2` | 구분선 위 0mm, 미주 사이 20mm, 구분선 아래 2mm |
| `2024-11-practice-above20-between0-below20` | 구분선 위 20mm, 미주 사이 0mm, 구분선 아래 20mm |
| `2024-11-practice-above20-between7-below2` | 구분선 위 20mm, 미주 사이 7mm, 구분선 아래 2mm |
| `2024-11-practice-no-separator-above20-between20-below20` | 구분선 없음, 구분선 위 20mm, 미주 사이 20mm, 구분선 아래 20mm |

## 분석 계획

1. `2024-11-practice-above20-between0-below20` sweep를 생성해 p10/p20 compare를 확인한다.
2. 문단 간 vpos rewind가 현재 단의 기존 bbox와 겹치는 경우를 render tree 기준으로 검출한다.
3. 단순히 모든 rewind를 넘기면 `shape987`이 23쪽으로 늘어난 경험이 있으므로, 다음 조건을 함께 본다.
   - 되감긴 paragraph가 visible text/equation을 갖는지
   - 되감김 후 예상 y가 현재 단의 마지막 visible bottom보다 충분히 위인지
   - 해당 paragraph가 빈 host 또는 picture-only anchor인지
4. `between=0`을 맞춘 뒤 `no-separator`와 `shape987` page count 회귀를 확인한다.

## 검증 대기

- `cargo fmt --all -- --check`
- `cargo build --bin rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage9_between0 --rhwp-bin target/debug/rhwp`
- page count: `2024-11-practice-shape987`, `2024-11-practice-no-separator-above20-between20-below20`, `2024-11-practice-above0-between20-below2`

## 2024-11 샘플 8종 sweep 확인

`--target`을 여러 번 지정하면 마지막 target만 실행되던 sweep 인자 처리를 고쳐, 8종을 한 번에
검증했다.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between7-below2 \
  --target 2024-11-practice-above0-between7-below20 \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage9_sample_check_all \
  --rhwp-bin target/debug/rhwp
```

결과:

| target | PDF | rhwp | 판정 |
|---|---:|---:|---|
| `2024-11-practice-shape987` | 21 | 22 | 1쪽 과다 |
| `2024-11-practice-above0-between0-below0` | 21 | 21 | 쪽수 일치, drift 후보 존재 |
| `2024-11-practice-above0-between7-below2` | 21 | 21 | 쪽수 일치, drift 후보 존재 |
| `2024-11-practice-above0-between7-below20` | 21 | 21 | 쪽수 일치, drift 후보 존재 |
| `2024-11-practice-above0-between20-below2` | 22 | 22 | 쪽수 일치, Stage8 20mm 보정 유지 |
| `2024-11-practice-above20-between0-below20` | 21 | 20 | 1쪽 과소 |
| `2024-11-practice-above20-between7-below2` | 21 | 21 | 쪽수 일치, frame overflow 없음 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23 | 21 | 2쪽 과소 |

우선순위:

1. `above20-between0-below20`: 구분선 위/아래 20mm는 반영되지만 `between=0`에서 후반 풀이가
   과소 예약되어 20쪽으로 끝난다.
2. `no-separator-above20-between20-below20`: 구분선은 그리지 않지만 위/아래/사이 20mm를 모두
   적용해야 하며, 현재 2쪽 과소다.
3. `shape987`: 기본 근방 9/8/7에서는 반대로 1쪽 과다라 큰 margin 계열 보정과 분리해야 한다.
