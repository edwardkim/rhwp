# Task #1105 최종 보고서 — sample16-hwp5 p21 page break 한컴 정합

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 구현 커밋: `d4587b27 fix: preserve hwp3 conversion page break`
- 상태: PR #1106 회수 완료, 문서화 후 PR 재준비

## 결과

`samples/hwp3-sample16-hwp5.hwp`에서 한컴 오피스와 다르게 `pi=440` 헤더가 이전 페이지 끝에 남던 문제를 정정했다.

```text
Before page 21: pi=426..440
After  page 21: pi=426..439

Before page 22: pi=441 table 이후
After  page 22: pi=440 "4. 서버통합..." 시작

전체 page count: 64 유지
```

## 변경 요약

1. HWP3-origin HWP5 변환본에서 line segment가 비어 있는 bridge 뒤의 page-reset 신호를 좁게 복원했다.
2. `PARA_LINE_SEG`가 없는 변환본 본문 문단의 synthetic line height를 compact하게 측정했다.
3. pagination, height measurement, 실제 layout이 같은 helper를 사용하도록 `corrected_line_height_for_variant_synthetic()`를 추가했다.
4. `tests/issue_1105.rs`를 추가해 page 21/22 경계를 고정했다.
5. #1086 회귀 테스트로 `k-water-rfp.hwp=27`, `hwpspec.hwp=178`, sample16-hwp5=64를 유지했다.

## 검증

```text
cargo fmt --all -- --check = pass
cargo test --test issue_1105 --test issue_1086 --test issue_1035_alignment --test issue_554 = pass
cargo test --test issue_nested_table_border = pass
git diff --check = pass
```

직접 덤프:

```text
page 21: 마지막 paragraph pi=439, pi=440 없음
page 22: pi=440, pi=441, pi=449 포함
```

## 절차 정리

PR #1106은 문서화 전에 생성되어 회수했다.

```text
PR #1106 = CLOSED
closedAt = 2026-05-24T08:02:44Z
```

새 PR 생성 또는 #1106 재오픈은 작업지시자 승인 후 진행한다. PR은 일반 PR로 생성하며 draft로 만들지 않는다.

## PR 본문 초안

```markdown
## Summary
- restore the Hancom page break before sample16-hwp5 section 4 when HWP3-origin conversion lost explicit break metadata
- use compact synthetic line height for HWP3-origin paragraphs without PARA_LINE_SEG across pagination, measurement, and layout
- add issue #1105 regression coverage for page 21/22 paragraph boundaries

## Tests
- cargo fmt --all -- --check
- cargo test --test issue_1105 --test issue_1086 --test issue_1035_alignment --test issue_554
- cargo test --test issue_nested_table_border
- git diff --check

Refs #1105
Stacked on #1103 until #1103 lands.
```
