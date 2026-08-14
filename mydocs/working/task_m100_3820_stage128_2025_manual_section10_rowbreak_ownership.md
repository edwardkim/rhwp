# Stage 128 - 2025 행정업무운영 편람 section 10 RowBreak 소유 복원

## 목표

Stage 127 커밋 `09aefdf6b`의 HWP 392쪽, HWPX 386쪽에서 시작해 HWP section 10에 남은 +9쪽을 줄인다. 이 Stage는 section 10의 첫 1x1 `RowBreak` 표와 그 직전 host flow의 page ownership을 대상으로 한다.

## 기준선

- Hancom PDF 및 `07555d200` HWP: 383쪽
- Stage 127 native HWP: 392쪽, section 10은 43쪽
- old renderer section 10: 34쪽
- Stage 126 이후 p4(`pi=4`)는 3 fragment지만 old renderer보다 시작 page가 한 쪽 늦다.

## 보존 계약

- Stage 126의 native HWP 실제 fragment flow와 HWP5-origin HWPX object-height advance 범위는 보존한다.
- Stage 127의 native HWP 2px 저장 행 반올림 수용은 보존한다.
- fixture 식별자나 문단 index가 아니라 RowBreak host/fragment의 저장 flow 근거로만 분기한다.

## 구현 순서

1. p4 직전 host item과 첫 fragment의 flow advance를 old/current dump로 대조한다.
2. 시작 page를 늦추는 object-height/host-flow 중복 소비를 본 작업 트리에서 최소 규칙으로 수정한다.
3. HWP/HWPX 전체 page count 및 p4 fragment cut을 기록한다.
4. build와 focused 회귀를 통과한 코드·결과 문서를 하나의 커밋으로 고정한다.

## 수용 기준

1. p4는 old renderer와 같은 page ownership에서 시작하며 fragment는 3개를 유지한다.
2. HWP 전체 쪽수는 392쪽보다 감소한다.
3. HWPX page count와 HWP5-origin HWPX regression은 유지된다.

## 구현

- `src/renderer/typeset.rs`에서 native HWP5의 비-TAC 1행 1열 `RowBreak` 표가 선언 높이 신뢰 상한을 넘으면, 선언 object height로 통째로 다음 page에 예약하지 않고 현재 host page에서 cell fragment scan을 시작하게 했다.
- 같은 형상에 셀 내부의 저장 `vpos` reset이 있으면 첫 fragment의 cut budget에만 `32px` tail allowance를 준다. 이 값은 cell-unit hard-break의 기존 frame-tail 임계와 같으며, continuation과 HWPX에는 적용하지 않는다.
- PageHide marker와 다음 page-break host를 병합하는 가설도 검토했으나, 기준 PDF physical page 278이 실제 blank page이고 page 279부터 footer가 다시 표시됨을 확인했다. 따라서 PageHide ownership을 바꾸는 코드는 채택하지 않았다.

## 결과

- native HWP p4(`pi=4`) cut은 `[] -> [29]`, `[29] -> [62]`, `[62] -> 끝`의 3 fragment가 됐다. Stage 127의 마지막 Q55 tail 전용 fourth fragment가 사라졌다.
- p4는 old renderer와 같은 HWP physical page 280에서 시작한다. 첫 fragment는 기준 PDF 목차 첫 page와 같이 제목 및 1-18번 항목을 함께 표시한다.
- 2025 행정업무운영 편람 native HWP: `392 -> 391`쪽, section 10: `43 -> 42`쪽이다.
- 같은 HWPX fixture는 `386`쪽으로 유지됐다.
- SVG 시각 대조에서 첫 fragment의 표 하단은 body area를 `5.7px` 넘지만 footer와 겹치지 않는다. 이 미세 하단 좌표 차이는 page count를 다시 늘리지 않는 범위에서 후속 stage가 다룬다.

## 검증

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo build --profile release-test
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_1891 --quiet
```

- build 통과
- `issue_1891`: 4 passed, 0 failed

## 잔여 과제

- 기준 PDF는 383쪽, native HWP는 391쪽으로 아직 `+8`쪽이다.
- p4 자체의 fragment 소유는 복원됐지만, 기준 PDF의 p4는 physical page 279-281이고 현재 native HWP는 280-282다. 이 한 page offset과 section 10 뒤의 추가 page 원인을 다음 stage에서 독립적으로 분석한다.
