# Task #1105 Stage 4 조사 기록 — sample16 2022 변환본 page break 재점검

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 기준 커밋: `23aa3f7f Task #1105: K-water 2024 page break 정합 보강`
- 요청: `hwp3-sample16-hwp5-2022.hwp`, `hwp3-sample16-hwp5-2018.hwp` 한컴오피스 정답지 대비 page break 재점검

## 1. 재현 명령

```bash
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16-hwp5-2018.hwp -p 4
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16-hwp5-2018.hwp -p 5
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16-hwp5-2022.hwp -p 4
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16-hwp5-2022.hwp -p 5
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16.hwp -p 4
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16.hwp -p 5
```

## 2. 관찰 결과

### HWP3 원본

- page 5: `pi=140`까지 배치
- page 6: `pi=141 "(4) 사업자 선정방식"`부터 시작
- `dump samples/hwp3-sample16.hwp -s 0 -p 141` 결과 `column_type=Page`, `vpos=0`

### 2018/2024 변환본

- page 5: `pi=140`까지 배치
- page 6: `pi=141 "(4) 사업자 선정방식"`부터 시작
- `pi=141`의 `LINE_SEG.vpos=0`
- 이 경계만 보면 HWP3 원본 기준과 일치한다.

### 2022 변환본

현재 rhwp 결과:

- page 5: `pi=141`, `pi=142 lines=0..1`까지 이전 페이지에 남음
- page 6: `pi=142 lines=1..3`부터 시작

관련 `LINE_SEG`:

```text
pi=140: vpos=57560, -13060
pi=141: vpos=-11108
pi=142: vpos=-9092, -7076, -5060
```

`pi=140` 둘째 줄부터 음수 좌표대가 이어져 cross-paragraph reset 판단이 깨진다. 현재 로직은
`prev_para.line_segs.last()`를 기준으로 직전 문단 끝을 판단하므로, `pi=140`의 마지막 줄
`-13060`이 직전 좌표로 들어가 `pi=141`을 새 페이지 시작으로 인식하지 못한다.

## 3. 원인 가설

Task #1042 Stage 5의 HWP3-origin HWP5 변환본 `line_segs.vpos` 정규화는 문단 단위로
`cumulative spacing_before`를 차감한다. 2022 변환본에는 문단 내부 또는 변환기 버전 차이로
음수 좌표대가 생기는 구간이 있으며, 이 값이 후속 page-reset 판단의 직전 좌표를 오염시킨다.

기존 #1105의 `pi=440` 보정은 line segment 누락 bridge와 `vpos=0` 근처 heading을 좁게 처리한다.
이번 2022 케이스는 `vpos=0`이 아니라 음수 좌표대가 이어지는 형태라 별도 가드가 필요하다.

## 4. 수정 후보

1. 회귀 테스트를 먼저 추가한다.
   - `samples/hwp3-sample16-hwp5-2022.hwp`
   - page 5에는 `pi=140`까지 있어야 한다.
   - page 5에는 `pi=141`이 없어야 한다.
   - page 6은 `pi=141`, `pi=142`, `pi=144`를 포함해야 한다.
   - 2018/2024는 이미 같은 경계가 맞으므로 회귀 가드로 함께 고정할 수 있다.

2. 음수 `vpos`가 직전 문단 끝 좌표를 오염시키지 않도록 reset 판단을 좁게 보강한다.
   - 대상은 `Document::is_hwp3_variant == true`인 HWP3-origin 변환본으로 제한한다.
   - 직전 문단의 마지막 줄이 음수이고 같은 문단 앞쪽에 큰 양수 `vpos`가 있으면, page-reset 판단에는 마지막 줄 대신 양수 쪽 최대 끝 좌표를 사용한다.
   - 현재 문단이 단일 `LINE_SEG` heading이고 first `vpos`가 음수이며, resolved spacing 기준
     `spacing_before >= 250 HU`, visible text일 때만 page break를 인정한다.

3. parser normalize 수정은 보류한다.
   - parser 단계에서 음수 좌표를 전역 보정하면 다른 변환본의 누적 정렬을 넓게 흔들 수 있다.
   - 이번 증상은 page break 판단에서 직전 음수 좌표를 사용한 것이 직접 원인이므로, 먼저 typeset/pagination의 좁은 가드가 안전하다.

## 5. 다음 단계

작업지시자 승인 후:

1. `tests/issue_1105.rs`에 sample16 2022 page break 회귀 테스트 추가
2. 필요 시 2018/2024 동일 경계 가드 추가
3. `src/renderer/typeset.rs`와 fallback `src/renderer/pagination/engine.rs`에 동일한 좁은 reset 판단 보강
4. 회귀 검증:

```bash
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_1035_alignment --test issue_1086 --test issue_554 --test issue_nested_table_border -- --nocapture
cargo fmt --all -- --check
git diff --check
```
