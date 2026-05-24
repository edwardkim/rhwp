# Task #1086 Stage 3 완료 보고서 — hwpspec.hwp 한컴 오피스 대조 재분석

- 이슈: [edwardkim/rhwp#1086](https://github.com/edwardkim/rhwp/issues/1086)
- 기준: 작업지시자 한컴 오피스 스크린샷 대조
- 입력: `samples/hwpspec.hwp`

## 1. 재현

작업지시자 스크린샷 기준 한컴 오피스는 `hwpspec.hwp` 를 178쪽으로 표시한다. 기존 rhwp 는 같은 위치에서 176쪽 또는 Stage 2 이후 177쪽에 머물렀고, 특히 20쪽에서 확장 컨트롤 그림(`s2:pi=89`)이 한컴보다 한 쪽 빨리 올라왔다.

핵심 차이:

```text
한컴 page 20: pi=78 표 후반 + pi=79..88 텍스트, pi=89 그림은 다음 쪽
rhwp 기존:     page 20 에 pi=89 그림과 3.4 문서 요약까지 포함
```

## 2. 구현

### 2.1 near-top vpos reset

HWP3-origin page tolerance 대상 문서에서 새 페이지 첫 문단이 `vpos=0` 대신 `200/500 HU` 근방으로 인코딩되는 케이스를 인정했다.

가드:

- `uses_hwp3_origin_page_tolerance()` 대상 문서에서만 활성화
- 그림-only 문단: `cv <= 200`, 직전 문단 하단 `> 52000 HU`
- 텍스트 문단: `cv <= 500`, 직전 문단 하단 `> 60000 HU`
- 표 host 문단은 제외해 `hwpspec pi=57`, `pi=78 -> pi=79` 정상 흐름 회귀를 차단

결과:

```text
page 20: pi=78 rows=19..26 + pi=79..88
page 21: pi=89 그림 시작
```

### 2.2 HWP3-origin title section filler

`hwpspec.hwp` Part II 시작 구역은 한컴이 직전 구역이 홀수 쪽에서 끝날 때 빈 쪽을 삽입해 다음 제목 페이지를 홀수 쪽에 맞춘다. 다음 패턴을 모두 만족할 때만 선행 빈 페이지를 삽입했다.

- HWP3-origin page tolerance 대상
- 이전 구역 마지막 쪽번호가 홀수
- 첫 문단에 `SectionDef flags & 0x3 == 0x3`
- 첫 문단에 header/page number hide
- 첫 문단에 전면 배경 그림 또는 Shape 그림
- 초반 명시 쪽나누기 존재

결과:

```text
page 66: 삽입된 빈 쪽, section=3 page_num=56
page 67: Part II 표지, page_num=57
page 69: "1. 개요", page_num=59
page 175: "변경 사항 이력", page_num=165
```

## 3. 회귀 확인

초기 near-top reset 은 일반 직접 작성 HWP인 `2022년 국립국어원 업무계획.hwp` 를 35쪽에서 36쪽으로 늘렸다. 이를 방지하기 위해 near-top reset 을 HWP3-origin page tolerance 대상 문서로 한정했다.

## 4. 최종 검증

직접 page count:

```text
samples/hwpspec.hwp = 178 pages
samples/2022년 국립국어원 업무계획.hwp = 35 pages
samples/k-water-rfp.hwp = 27 pages
samples/aift.hwp = 74 pages
```

자동 검증:

```text
cargo fmt --all -- --check = pass
cargo build --release --bin rhwp = pass
cargo clippy --release --lib -- -D warnings = pass
cargo test --release --lib = pass (1352 passed, 6 ignored)
cargo test --release --test issue_1086 --test issue_554 = pass
cargo test --release --test issue_nested_table_border --test svg_snapshot = pass
```
