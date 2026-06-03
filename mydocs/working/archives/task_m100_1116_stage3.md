# Task #1116 Stage 3 보고서 — 3mm 격자 기준 재측정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 3mm 격자 산출물 생성 및 현재 좌표 재측정

## 1. 배경

작업지시자 지적:

```text
아직 안맞는데?
```

Stage 2는 p2 목차 leader가 페이지 번호 뒤로 넘어가는 문제만 잡았다. 작업지시자 캡처 기준의 전체 목차/본문 배치 정합은 아직 해결되지 않았다. 또한 기존 `--show-grid`는 1mm 격자 고정이라, 한컴 3mm 격자 캡처와 직접 비교하기에 부적절했다.

## 2. CLI 보강

`export-svg`에 `--show-grid=3mm` 형식을 추가했다.

기존:

```bash
--show-grid
```

- 1mm 격자 유지.

추가:

```bash
--show-grid=3mm
```

- 3mm 격자.
- SVG pattern 확인값: `width=11.3386 height=11.3386`.

## 3. 3mm 기준 SVG 산출물

p2:

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm \
  -p 1 \
  --show-grid=3mm \
  --debug-overlay \
  --show-control-codes
```

산출물:

```text
output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm/hwp3-sample16-hwp5_002.svg
```

p3:

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm \
  -p 2 \
  --show-grid=3mm \
  --debug-overlay \
  --show-control-codes
```

산출물:

```text
output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm/hwp3-sample16-hwp5_003.svg
```

## 4. p2 목차 현재 좌표

3mm 격자 간격:

```text
11.3386 px
```

주요 overlay 좌표:

| 항목 | pi | SVG y(px) | 3mm 칸수 |
|------|---:|----------:|---------:|
| `목 차` | 25 | 75.6 | 0.00 |
| `I. 사업개요` | 26 | 126.8 | 4.52 |
| `1. 추진목적` | 27 | 152.9 | 6.81 |
| `2. 추진방향` | 28 | 175.4 | 8.80 |
| `3. 주요 추진내용` | 29 | 198.0 | 10.79 |
| `II. 제안 일반사항` | 31 | 230.9 | 13.70 |
| `IV. 프로젝트 과업범위` | 44 | 506.8 | 38.03 |
| `VII. 한국수자원공사 일반현황` | 62 | 892.0 | 72.00 |
| `별첨1` | 67 | 996.1 | 81.18 |
| `별첨2` | 68 | 1020.4 | 83.32 |

Stage 2 보정 후 leader:

```text
leader count = 28
max x2 = 624.5
page number x range = 609.7..661.8
```

즉 leader가 페이지 번호 뒤로 길게 넘어가는 결함은 줄었지만, 목차 전체의 x/y 배치와 항목 들여쓰기/페이지 번호 열은 한컴 3mm 캡처 기준으로 다시 판정해야 한다.

## 5. p3 본문 현재 좌표

주요 overlay 좌표:

| 항목 | pi | SVG y(px) | 3mm 칸수 |
|------|---:|----------:|---------:|
| `I. 사업개요` | 69 | 75.6 | 0.00 |
| `1. 추진목적` | 70 | 124.0 | 4.26 |
| 목적 박스 host | 71 | 282.6 | 18.25 |
| 빈 문단 | 72 | 293.0 | 19.17 |
| `2. 추진방향` | 73 | 299.2 | 19.72 |
| 본문 (1) | 74 | 333.3 | 22.73 |
| 본문 (2) | 75 | 414.0 | 29.84 |
| 본문 (3) | 76 | 494.6 | 36.95 |
| 본문 (4) | 77 | 575.3 | 44.06 |
| 빈 문단 | 78 | 655.9 | 51.17 |
| `3. 주요 추진내용` | 79 | 662.1 | 51.72 |
| BCP 항목 | 80 | 696.2 | 54.73 |
| 첫 체크 항목 | 81 | 723.1 | 57.10 |
| 마지막 체크 항목 | 86 | 861.8 | 69.33 |

plain SVG 기준 `3. 주요 추진내용` 이하 줄 텍스트:

```text
710.9 (1)공사비상대응체계(BCP)및목표시스템아키텍쳐분석․설계
737.8 □공사현행운영체계의위험요소분석․진단을통한추진전략수립
765.6 □서버통합및원격지재해복구센터의목표수준정의
793.3 □공사정보처리연속성확보를위한비상대응체계(BCP:BusinessContinuityPlanning)수립
821.0 □서버통합및재해복구센터목표시스템아키텍쳐설계․확정
```

## 6. 현재 판단

1. `--show-grid=3mm` 없이 만든 SVG는 한컴 3mm 격자 캡처와 직접 비교하지 않는다.
2. p2는 leader 뒤넘김만 줄었고, 전체 목차 배치 정합은 아직 남아 있다.
3. p3는 목적 박스 주변 한 구간만 볼 것이 아니라 `2. 추진방향` 본문 4개 문단과 `3. 주요 추진내용` 이후 체크 항목까지 누적 높이를 봐야 한다.
4. Stage 2에서 추가한 p3 위치 가드는 "현재 위치 고정"에 가까워 실제 정답 기준으로는 부족하다. Stage 4에서 한컴 3mm 캡처 기준 수치가 나오면 테스트 기대값을 조정해야 한다.

## 7. 다음 보정 후보

p2:

- 페이지 번호 열 x 위치가 한컴 기준보다 오른쪽/왼쪽인지 확인.
- 상위 항목과 하위 항목의 들여쓰기 차이 확인.
- leader 시작점이 항목 텍스트 직후에서 시작하는지, 불필요하게 앞/뒤로 붙는지 확인.

p3:

- `pi=74..77` 3줄 본문 문단의 line advance가 한컴보다 짧은지/긴지 확인.
- `pi=80..86` 체크 항목 문단의 줄 간격과 체크박스 baseline 확인.
- `Business Continuity Planning` 포함 줄의 글자 폭/공백 처리도 함께 확인.

## 8. 검증

실행 완료:

```bash
cargo build --bin rhwp
cargo fmt --all -- --check
git diff --check
```
