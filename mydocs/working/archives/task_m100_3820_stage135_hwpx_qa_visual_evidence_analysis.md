# Stage 135 - 2025 편람 HWPX Q&A 시각 증적 분석

## 목표

Stage 134에서 2025 행정업무운영 편람의 HWPX source와 HWP 저장-재로드를 Hancom PDF와 같은 383쪽으로 맞추고, Q8 표제를 physical p285에 고정했다. 이번 Stage는 p284~p285의 page owner가 같다는 사실만으로 시각 정합을 추정하지 않고, PDF·native HWP·HWPX의 표 테두리, Q7 response tail, Q8 표제와 여백을 같은 좌표계에서 대조한다.

## 기준 자료

- PDF oracle: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` (383쪽, 재생성 금지)
- native 입력: `samples/2025 행정업무운영 편람(최종).hwp`
- HWPX 입력: `samples/2025 행정업무운영 편람(최종).hwpx`
- 선행 커밋: `511efea1e` (`fix: HWPX Q&A 쪽 소유권을 PDF와 맞춘다`)

## 분석 범위

1. PDF p284~p285와 native HWP/HWPX의 대응 SVG를 같은 DPI로 얻는다.
2. Q7의 마지막 response 문단, p284 하단 여백, p285의 Q7 tail 및 Q8 표제의 bbox·table border를 대조한다.
3. HWPX source와 HWP 저장-재로드 SVG의 차이도 함께 확인해 직렬화 경계의 회귀를 분리한다.
4. 첫 차이가 발견되면 raw lineSeg, row cut, render tree 중 어느 계층의 결함인지 근거를 남긴다. 구현은 그 분석 뒤에만 시작한다.

## 보존 계약

- HWPX source와 저장-재로드의 383쪽 및 Q8 physical p285 owner를 후퇴시키지 않는다.
- native HWP의 383쪽 및 p285 Q8 owner를 변경하지 않는다.
- PDF oracle과 기존 Stage 134 증적을 대체하거나 재생성하지 않는다.
- fixture 경로, physical page 번호, paragraph index만으로 구현을 분기하지 않는다.

## 완료 기준

1. p284~p285의 PDF/native/HWPX 비교 산출물과 페이지 대응표가 남는다.
2. first visual divergence가 없으면 동일하다는 비교 근거를 남긴다.
3. 차이가 있으면 재현 가능한 source topology와 후속 구현 범위를 확정한다.

## 분석 결과

### PDF/native와 달랐던 최초 HWPX fragment

PDF p284와 native HWP p284는 Q5 응답의 둘째 줄 `않는 내부결재문서 외에는 ...`부터 시작한다. 초기 HWPX p284에는 그보다 앞선 첫 줄 `문서는 결재권자의 결재가 완료된 시점에 ...`까지 남아 있었다. 이 한 줄 때문에 이후 좌표가 정확히 한 line pitch인 `24.50px`씩 늦어졌다.

| p284 text baseline | native HWP | 초기 HWPX | 차이 |
| --- | ---: | ---: | ---: |
| Q6 표제 | 516.92px | 541.43px | +24.51px |
| Q6 첫 response | 586.54px | 611.04px | +24.50px |
| Q7 표제 | 769.81px | 794.31px | +24.50px |
| Q7 첫 response | 843.79px | 868.29px | +24.50px |

Q5 표는 non-TAC `RowBreak` 6행×5열/15-cell, declared height `11382 HU`, outer bottom margin 0의 마지막 응답 행이다. target response cell은 3문단/`3·5·3` lineSeg이며 첫 문단이 `vertpos=0 -> 1838 -> 0` frame reset을 갖는다. 이는 Stage 134의 Q7(5문단)과 다른 stored-frame topology다.

### 구현 결정

1. HWPX Q5 topology에는 row-cut 예산 `16px`과 physical tail tolerance `32px`를 분리해 적용했다. 16px은 첫 response line만 선택한다.
2. 이 조각의 content height는 `24.5px`로 일반 orphan 최소값 `25px`보다 0.5px 작다. saved-frame Q5 topology에만 painted-height(visible cell padding 포함) 기준을 적용해 통째 이월을 막았다.
3. Q5 보정 뒤에는 기존 HWPX Q7의 96px allowance가 다음 response line을 p284 body clip 밖으로 과수용했다. Q7 raw budget 18.3px에 필요한 stored cut 82.9px을 맞추도록 HWPX allowance를 `65px`로 축소했다.
4. 16px Q5 allowance만 적용하거나 Q7을 96px으로 유지하는 실험은 각각 owner 미변경 또는 Q8 p285의 24.51px 조기 배치를 만들었으므로 폐기했다.

### 최종 시각 증적

현재 HWPX와 native HWP의 SVG text baseline은 다음과 같이 같다.

| text | native HWP | HWPX | 차이 |
| --- | ---: | ---: | ---: |
| p284 Q6 표제 | 516.92px | 516.92px | 0.00px |
| p284 Q6 첫 response | 586.54px | 586.54px | 0.00px |
| p284 Q7 표제 | 769.81px | 769.81px | 0.00px |
| p284 Q7 첫 response | 843.79px | 843.79px | 0.00px |
| p285 Q8 표제 | 561.64px | 561.64px | 0.00px |

PDF p284/p285와 native/HWPX SVG를 같은 96 DPI raster로 대조했다. PDF와 native가 공유하는 Q5 p283→p284 line owner 및 Q8 p285 table 시작을 HWPX도 동일하게 보존한다. PDF oracle은 기존 `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`(383쪽)를 그대로 사용했고 재생성하지 않았다.

## 회귀 및 검증 결과

`tests/issue_3930_hwpx_hwp_save_layout.rs`는 다음을 고정한다.

- HWPX Q5 saved-frame 첫 response line은 source p283에 있고 p284에는 없다.
- HWPX source 및 HWP 저장-재로드가 p283/p284/p285 render tree를 동일하게 보존한다.
- HWPX Q8 표제는 source 및 저장-재로드 모두 p285에 있다.
- HWPX source/저장-재로드와 native HWP 모두 Hancom PDF와 같은 383쪽이다.

```text
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet

running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
