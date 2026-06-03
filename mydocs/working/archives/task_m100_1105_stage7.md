# Task #1105 Stage 7 완료 보고서 — HWP3 원본 글머리표 정답지 정합

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 기준: 한컴오피스 정답지 / HWP3 원본 표시

## 1. 추가 피드백

작업지시자 확인 결과 `hwp3-sample16.hwp` 원본의 23쪽 글머리표가 한컴오피스 정답지와 다르게 보였다.

- rhwp: `○`
- 한컴오피스 정답지: `□`

같은 문단군의 HWP5 변환본은 첫 글자가 `U+F03C5`이고, 렌더러가 이를 `□`로 매핑해 이미 정답지와 맞았다.

## 2. 원인

HWP3 파서의 사적 문자 매핑에서 `0x3366`을 `U+25CB`(`○`)로 직접 낮추고 있었다.

과거에는 `U+F03C5` PUA 글리프가 폰트 폴백에서 보이지 않는 문제가 있어 `○`로 치환했지만,
현재 렌더러는 `U+F03C5`를 `□`로 확장한다.

따라서 HWP3 파서에서 `0x3366`을 `○`로 낮추는 우회가 오히려 HWP3 원본만 한컴오피스 정답지와 다르게
보이게 만든 원인이었다.

## 3. 구현

`src/parser/hwp3/johab.rs`

- `0x3366 -> U+25CB` 매핑을 제거
- `0x3366 -> U+F03C5`로 PUA 보존
- 표시 값은 공통 렌더러의 `U+F03C5 -> □` 매핑을 사용

`tests/issue_1105.rs`

- HWP3 원본 `hwp3-sample16.hwp` page 23 회귀 테스트 추가
- parser IR에서는 `U+F03C5`가 보존되는지 확인
- 렌더 결과에는 `□`가 포함되고 `○ 계약상대자는` 패턴이 남지 않는지 확인

## 4. 확인

`hwp3-sample16.hwp` `pi=450` dump 결과:

```text
텍스트: "󰏅 계약상대자는 ..."
[CS] pos=0 ... char="\u{f03c5}"
```

23쪽 page dump:

```text
FullParagraph  pi=450 ... "󰏅 계약상대자는 ..."
FullParagraph  pi=460 ... "󰏅 계약상대자는 ..."
pi=461 없음
```

## 5. 검증

```bash
cargo test --test issue_1105 -- --nocapture
```

결과:

- `issue_1105`: 13 passed
