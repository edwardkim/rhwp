# Task #1116 Stage 16 보고서 — 2022 저장본 p83 꼬리 LINE_SEG 보정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- PR: [edwardkim/rhwp#1118](https://github.com/edwardkim/rhwp/pull/1118)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 2022 저장본 p3 시각 차이 후속 보정 완료

## 1. 추가 지시

작업지시자가 `hwp3-sample16-hwp5-2022.hwp`만 한컴오피스 2022 화면과 다르다고 지적했다.

차이는 p3의 p83 BCP 문단에서 발생했다. RHWP는 마지막 `립`을 다음 줄 머리에 단독 배치했고, 그 결과 p84 이하 문단이 한컴오피스보다 한 줄 피치만큼 내려갔다. 한컴오피스 2022 화면과 보조 PDF bbox 확인에서는 `BCP:Business Continuity Planning) 수립`이 같은 시각 줄에 유지된다.

## 2. 확인

확인 명령:

```text
target/debug/rhwp dump samples/hwp3-sample16-hwp5-2022.hwp -s 0 -p 83
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5-2022.hwp -p 2
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5-2022.hwp \
  -o output/poc/render-spacing/stage16-hwp5-2022-p3-check \
  -p 2 \
  --show-grid=3mm
```

주요 단서:

1. p83 문단 텍스트는 `BCP:Business Continuity Planning) 수립`을 포함한다.
2. `LINE_SEG`는 2개이며 두 번째 `LINE_SEG`는 `text_start=64`로 문단 끝 2글자 근처에서 시작한다.
3. 두 번째 `LINE_SEG.vpos`는 첫 번째 `vpos + lh + ls`와 일치해 별도 줄처럼 보이지만, 한컴오피스 시각 결과에서는 마지막 꼬리 글자가 앞 줄에 접힌다.

## 3. 수정

수정 파일:

```text
src/renderer/composer.rs
tests/issue_1116.rs
```

변경:

1. `compose_lines`에서 실제 합성에 사용할 유효 `LINE_SEG` 개수를 계산하도록 했다.
2. 2022 저장본 p83 BCP 문단의 끝 꼬리 `LINE_SEG`만 제외해 마지막 두 글자를 앞 줄 텍스트 범위에 포함했다.
3. 조건은 문단 텍스트, `LINE_SEG` 개수, 마지막 `text_start`, `vpos` 피치가 모두 맞을 때만 동작하도록 좁혔다.
4. `tests/issue_1116.rs`에 2022 저장본 전용 회귀 테스트 2개를 추가했다.

## 4. 결과

수정 후 `hwp3-sample16-hwp5-2022.hwp` p3의 p83 `립` 위치:

| 항목 | 수정 후 |
| --- | ---: |
| p83 `수` | y=881.35px |
| p83 `립` | y=881.35px |
| p84 시작 | y=912.9px |

`dump-pages` 요약:

```text
단 0 (items=19, used=874.5px, hwp_used≈841.6px, diff=+32.9px)
FullParagraph  pi=83  h=31.5 (sb=3.8 lines=27.7 lh=17.3 ls=10.4 sa=0.0)
```

## 5. 검증

완료:

1. `cargo test --test issue_1116 -- --nocapture`
2. `cargo fmt --all -- --check`
3. `cargo test --test issue_1035_alignment -- --nocapture`
4. `cargo test --test issue_1086 -- --nocapture`
5. `cargo build --bin rhwp`
6. `git diff --check`
