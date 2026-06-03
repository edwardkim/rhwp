# Task #1105 Stage 1 완료 보고서 — 재현, 절차 회수, 원인 정리

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 관련 PR: #1106 회수 완료

## 1. 절차 회수

문서화/승인 전에 구현 커밋과 PR 생성이 먼저 진행되는 절차 오류가 있었다.

회수 조치:

```text
PR #1106 = CLOSED
closedAt = 2026-05-24T08:02:44Z
사유 = 하이퍼-워터폴 문서화 후 재준비
```

원격 브랜치 `origin/local/task1105`는 남겨두되, 새 PR 생성은 문서화 완료와 작업지시자 승인 후 진행한다.

## 2. 한컴/rhwp 차이

작업지시자 스크린샷 기준 한컴은 `hwp3-sample16-hwp5.hwp` p21 끝에 `4. 서버통합...` 헤더를 넣지 않는다. 다음 페이지가 해당 헤더로 시작한다.

rhwp 기존 문제:

```text
page 21: pi=426..440
page 22: pi=441 table 이후 본문
```

목표:

```text
page 21: pi=426..439
page 22: pi=440, pi=441, pi=442..449
전체 64 pages 유지
```

## 3. 원인 분석

`pi=440` 자체에는 `vpos=852`가 있어 page-reset 신호가 남아 있다. 그러나 직전 real LINE_SEG 문단과 `pi=440` 사이에 `LINE_SEG`가 없는 본문 문단들이 끼어 있어 기존 cross-paragraph reset 조건이 보수적으로 동작하지 않았다.

과거 #1035에서 aux trigger를 단순 적용하면 `64 -> 65 pages`가 되었기 때문에, page break 복원만으로는 충분하지 않았다.

over-split 원인:

- `pi=442` 이후 PUA bullet 본문 문단은 `PARA_LINE_SEG`가 없다.
- composer fallback은 synthetic line을 만든다.
- 기존 높이 보정은 raw line height가 폰트보다 작으면 ParaShape `160%`를 다시 적용한다.
- HWP3-origin HWP5 변환본에서는 이 합성 높이가 한컴보다 크고, force-break 후 새 페이지에서 65쪽으로 불어난다.

## 4. 구현 방향

1. page-reset은 bridge missing line segment 패턴이 있는 경우에만 좁게 복원한다.
2. HWP3-origin HWP5 변환본의 synthetic line height는 `max_fs`로 compact하게 측정한다.
3. Typeset, HeightMeasurer, LayoutEngine 모두 같은 helper를 사용해 pagination과 실제 SVG 레이아웃 drift를 막는다.
4. `tests/issue_1105.rs`로 p21/p22 경계를 직접 고정한다.

## 5. 다음 단계

Stage 2에서 구현 커밋 `d4587b27`의 내용을 문서화하고 자동 검증 결과를 정리한다.
