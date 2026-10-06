# Stage 131 - 2025 행정업무운영 편람 Q&A terminal spacer 분석

## 목표

Stage 130 커밋 `63cc6caab` 뒤 native HWP 386쪽과 Hancom PDF 383쪽 사이에 남은 `+3`쪽의 source owner를 분해한다. 이번 Stage는 Stage 130의 Q&A pre-defer bypass를 넓히지 않고, terminal spacer와 후속 paragraph가 만드는 물리 page를 분석한다.

## 기준선

- Hancom PDF: 383쪽
- Stage 130 native HWP: 386쪽
- Stage 130 HWPX: 386쪽
- Stage 130은 6x5 native HWP Q&A `RowBreak` 표를 declared/saved-frame pre-defer에서 제외해 기존 fragment scanner로 전달했다.
- focused regression: `tests/issue_3930_hwpx_hwp_save_layout.rs`, native HWP 386쪽 및 HWPX 386쪽.

## 남은 누적 분기

Stage 130 `dump-pages`의 historical/current first owner 대조에서 final `+3`은 다음 세 경계에서 각각 한 쪽씩 증가한 뒤 문서 끝까지 유지된다.

| 경계 | historical first owner | Stage 130 first owner | 누적 차이 |
| --- | ---: | ---: | ---: |
| `pi=037` 뒤 `pi=042` | p295 | p296 | +1 |
| `pi=056` 뒤 `pi=057` | p297 | p299 | +2 |
| `pi=074` 뒤 `pi=075` | p301 | p304 | +3 |

`pi=085`의 일시 `+4`는 후속 `pi=098` owner 합류 뒤 다시 `+3`이므로 이 Stage의 영구 차이 후보가 아니다.

## 분석 순서

1. 세 경계의 old/current page item sequence와 source `LINE_SEG` vpos를 대조한다.
2. terminal row가 빈 spacer인지, visible response tail인지, 또는 다음 host paragraph의 page rewind인지 구분한다.
3. historical renderer가 같은 suffix page에 후속 표 또는 빈 host를 어떻게 배치했는지 확인한다.
4. 분석 근거가 충분할 때만 scanner의 terminal-spacer owner 또는 saved host-line advance를 최소 범위로 수정한다.

## 분석 결과

### `pi=039`: 중복 빈 tail guide

- `pi=037`은 native HWP의 비글자처럼-취급 6행×5열, 15-cell `RowBreak` Q&A 표다.
- 뒤의 `pi=038`과 `pi=039`는 모두 control 없는 빈 문단, `ps_id=19`, `lh=1000 HU`, `ls=600 HU`다.
- 두 번째 줄의 `vpos=53308`은 첫 줄 `vpos=51708`에 정확히 `1000+600 HU`를 더한 값이다.
- historical renderer는 첫 빈 줄만 같은 suffix page에 두었다. Stage 130은 두 번째 guide까지 flow로 계상해 `pi=040`을 한 page 뒤로 민다.
- 따라서 첫 줄은 보존하고, 위 저장 사다리를 만족하는 둘째 줄만 `HiddenEmptyPara`로 기록한다.

### `pi=056`, `pi=074`: terminal spacer 행

- 두 표는 모두 native HWP 비글자처럼-취급 6행×5열, 15-cell `RowBreak` Q&A 표이며 마지막 `r=5`의 세 셀이 전부 빈 `row_span=1` spacer다.
- `pi=056`의 마지막 행 선언 높이는 `2182 HU`, trace상 현 fragment에서 이 행까지 포함하면 `51.7px` 초과한다. `pi=074`는 `1849 HU`, `60.4px` 초과한다.
- 기존 모든 RowBreak 표 공통 허용치 `40px`보다 커 두 spacer가 각각 `37.3px`, `24.7px`짜리 continuation-only page를 만든다.
- 두 표에는 `outer_margin_bottom=566 HU (2mm)`가 있으나, 일시 차이만 만들고 후속 owner에서 회복되는 `pi=085`는 `outer_margin_bottom=0`이다.

## 구현

- 일반 `RowBreak` 표의 `40px` trailing-spacer 허용치는 변경하지 않는다.
- native HWP, 비글자처럼-취급, 6행×5열/15-cell Q&A, `outer_margin_bottom>0`인 표의 마지막 빈 spacer 행에만 `64px` 허용치를 적용한다.
- `pi=085`와 HWPX/TAC/일반 표는 기존 경로를 유지한다.
- 위 중복 guide predicate는 source paragraph index나 physical page 번호를 사용하지 않는다.

## 검증 결과

- 실행: `CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout --quiet`
- 결과: 3 passed, 0 failed (0.93초).
- native HWP `samples/2025 행정업무운영 편람(최종).hwp`: 386쪽에서 **383쪽**으로 감소해 Hancom PDF 383쪽과 일치한다.
- HWPX `samples/2025 행정업무운영 편람(최종).hwpx`: **386쪽** 기준선을 유지한다.
- 이 Stage는 source 구조가 증명한 두 terminal spacer와 한 중복 tail guide만 바꾸며, `pi=085`의 일시적인 owner 차이는 의도적으로 건드리지 않았다.

## 보존 계약

- PDF physical 278의 PageHide blank page와 Stage 128의 1x1 목차 표 stored-frame tail은 유지한다.
- Stage 129의 96px short-tail guard와 Stage 130의 Q&A pre-defer bypass를 회귀시키지 않는다.
- HWPX, TAC, 글자처럼 취급되는 표, 일반 `RowBreak` 표에는 적용하지 않는다.
- fixture 이름, physical page 번호, paragraph index로 코드 분기하지 않는다.

## 수용 기준

1. 남은 세 increment의 source owner와 historical/current item sequence를 결과 문서에 기록한다.
2. 실제 terminal-spacer 또는 saved source signal이 증명하는 경계만 구현한다.
3. native HWP page 수와 HWPX 386쪽 regression 결과를 코드 변경 뒤 기록한다.
