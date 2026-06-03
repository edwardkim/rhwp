# Task #1105 Stage 5 완료 보고서 — sample16 변환본 page break 후속 보정

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 기준: 한컴오피스/HWP3 원본 page break

## 1. 구현 내용

### 1.1 회귀 테스트 추가

`tests/issue_1105.rs`에 sample16 변환본 4종 경계 가드를 추가했다.

- `hwp3-sample16-hwp5-2010.hwp`
- `hwp3-sample16-hwp5-2018.hwp`
- `hwp3-sample16-hwp5-2022.hwp`
- `hwp3-sample16-hwp5-2024.hwp`

공통 단언:

```text
page 4: pi=118 포함
page 5: pi=119 부터 시작, pi=118 없음
page 5: pi=140 까지
page 6: pi=141, pi=142, pi=144 포함
page_count == 64
```

### 1.2 음수 vpos reset 보정

`hwp3-sample16-hwp5-2022.hwp`는 다음 패턴으로 page break 신호가 음수 좌표대에 걸쳐 있었다.

```text
pi=140: vpos=57560, -13060
pi=141: vpos=-11108
```

기존 판단은 `pi=140` 마지막 줄의 음수 좌표를 직전 끝 좌표로 써서 `pi=141` 앞 break를 놓쳤다.

보정 조건:

- 직전 문단 마지막 real `LINE_SEG`가 음수
- 같은 직전 문단에 큰 양수 `vpos` line이 존재
- 현재 문단은 단일 `LINE_SEG`
- 현재 문단 first `vpos`가 음수
- 현재 문단은 visible text heading이며 control 없음
- resolved spacing 기준 `spacing_before >= 250 HU`

### 1.3 하단 단일 제목 reset 보정

`hwp3-sample16-hwp5-2010.hwp`는 음수 좌표가 아니라, page break 대상 제목이 이전 페이지 하단에
단일 `LINE_SEG`로 남고 다음 본문 문단이 다시 페이지 상단 좌표대로 돌아가는 패턴이었다.

```text
pi=140: vpos=55868, 57820
pi=141: vpos=59772
pi=142: vpos=2584, 4600, 6616
```

HWP3 원본과 한컴오피스 정답지 기준으로는 `pi=141 "(4) 사업자 선정방식"`이 다음 페이지 첫 문단이다.
따라서 현재 문단이 하단 단일 heading이고, 바로 다음 real 문단의 first `vpos`가 상단 좌표대로
작아지는 경우에도 좁게 page reset으로 인정했다.

보정 조건:

- 현재 문단은 직전 real 문단 바로 다음 문단
- 현재 문단은 단일 `LINE_SEG`
- 현재 문단 first `vpos`가 body 하단 75% 이후
- 다음 real 문단 first `vpos`가 양수이고 상단 좌표대
- 현재 문단은 visible text heading이며 control 없음
- resolved spacing 기준 `spacing_before >= 250 HU`

### 1.4 page 5 시작 지연 reset 보정

`hwp3-sample16-hwp5-2022.hwp`는 `pi=118`이 `vpos=0`으로 인코딩되어 일반 `vpos reset`
가드가 이를 page break로 오인했다. 한컴오피스 정답지 기준으로는 `pi=118`이 page 4 끝에
남고, `pi=119`가 page 5 첫 문단이어야 한다.

```text
pi=117: vpos=52472
pi=118: vpos=0
pi=119: vpos=1692, 3772, 5852
```

보정 조건:

- 현재 문단은 낮은 spacing의 단일 `vpos=0` 본문 문단
- 직전 real 문단 끝이 body 하단 70% 이후
- 다음 문단은 visible text heading이며 control 없음
- 다음 문단은 `LINE_SEG` 2개 이상, first `vpos`가 0이 아닌 상단 좌표대
- 다음 문단 resolved spacing 기준 `spacing_before >= 500 HU`

따라서 `pi=118` 앞에서는 일반 reset을 억제하고, 바로 다음 `pi=119` 앞에서 지연 page reset을
인정한다. `hwp3-sample-hwp5.hwp`의 16쪽 회귀를 막기 위해 suppression은 다음 heading 패턴이
확인되는 경우에만 적용한다.

적용 파일:

- `src/renderer/typeset.rs`
- `src/renderer/pagination/engine.rs`

## 2. 검증

```bash
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_1035_alignment --test issue_1086 --test issue_554 --test issue_nested_table_border -- --nocapture
cargo fmt --all -- --check
git diff --check
```

결과:

- `issue_1105`: 8 passed
- `issue_1035_alignment`: 4 passed
- `issue_1086`: 4 passed
- `issue_554`: 12 passed
- `issue_nested_table_border`: 2 passed
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed

## 3. 확인 결과

`hwp3-sample16-hwp5-2010.hwp`:

```text
page 5: ... pi=140
page 6: pi=141, pi=142, pi=144 ...
page_count: 64
```

`hwp3-sample16-hwp5-2022.hwp`:

```text
page 4: ... pi=118
page 5: pi=119 ... pi=140
page 6: pi=141, pi=142, pi=144 ...
page_count: 64
```

`hwp3-sample16-hwp5-2018.hwp`, `hwp3-sample16-hwp5-2024.hwp`:

```text
page 5: ... pi=140
page 6: pi=141, pi=142, pi=144 ...
page_count: 64
```
