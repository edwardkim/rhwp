---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7319 검토 기록 — HWPX 홀수 문단 여백의 `unit="CHAR"` 보존

- 원 PR: [#7319](https://github.com/edwardkim/rhwp/pull/7319)
- 관련 이슈: [#6875](https://github.com/edwardkim/rhwp/issues/6875) — **계속 열어 둠**
- 작성자: `planet6897` (기존 기여자)
- 원 code head: `3ff44327b037170b254fa517b996285aac8bf903`
- 통합 검토 branch: `review/open-prs-20260921`
- 적용 commit: `760cf36fcc7992d0c225603827358e02d977c1e5` (`cherry-pick -x`, 충돌 없음)
- 작성 시점 원 PR 상태: `MERGEABLE / CLEAN`, 원 head CI 성공

## 변경과 범위

`hp:case`의 문단 여백·간격 자식에서 저장 HWP unit 값이 홀수이면, 절반값과
`unit="CHAR"`를 쓴다. 파서는 같은 표시를 `case * 2 + 1`로 복원한다. 짝수는 기존
`HWPUNIT` 표현을 유지한다.

이는 HWPX serializer/parser 구조 보존 변경이며 renderer·layout·paint 경로는 바꾸지 않는다.
Visual Sweep은 비대상이다.

## 구현·검증 근거

- `write_margin_child`는 홀수/짝수와 `half` case를 분리해 `stored = case * 2 (+1 when CHAR)`
  역관계를 보존한다. 음수 홀수도 Rust의 0 방향 나눗셈과 같은 역식으로 되돌아간다.
- `tests/cases/issue_6875_hwpx_char_unit_margin_roundtrip.rs`의 신규 2건은 저장소 실물 HWP로
  CHAR 표기 규칙과 HWP → HWPX → HWPX 재파싱 여백 등식을 각각 확인한다.
- 실제 검증 입력: `samples/issue5714/1490000-200800034_vietnam_labor_report.hwp`
  (저장소 기존 commit, SHA-256
  `da3550d9f370b52823bd63eae0431d1b86fae46ae211b9cb2cd987165a8c0904`).
  테스트 안에서 생성·소비한 HWPX는 별도 fixture가 아니다.
- integration head `bd974e19b`에서 `cargo nextest run --cargo-profile release-test -j 8
  --test regression_suite_006 -E 'test(issue_6875_hwpx_char_unit_margin_roundtrip)'` — **2 passed**.

## 검증 범위의 한계

원 PR이 인용한 07939 코퍼스 HWP, 한컴 변환 HWPX, PDF는 이 검토 대상 commit에 없다.
따라서 이 검토는 그 문서의 `545 → 558`쪽 수치나 한컴 출력 비교를 재검증한 것이 아니다.
그 주장은 통합 PR의 수용 근거나 merge 후 comment에 쓰지 않는다. 저장소 fixture에서 직접
확인한 serializer 계약만 수용 범위이며, #6875의 다른 경로는 미해결로 남는다.

## 공통 조판 원칙과 입력 보존

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 홀수 bit를 `CHAR`로 명시하는 역식이 serializer·parser에 대칭으로 구현됨 |
| 측정·배치, 분할·이어받기, 줄 소속 | 비해당 | renderer/typeset 경로를 변경하지 않음 |
| 사례와 증거의 독립성 | 충족 | 저장소 HWP fixture의 출력 XML 계약을 직접 확인 |
| 기준값 변경 | 비해당 | baseline/golden 변경 없음 |
| 검증 입력 commit 포함 | 충족 | 위 HWP fixture는 추적된 동일 파일이며 실제 명령도 저장소 경로를 사용 |

## 최종 판정

**승인.** 단, 수용 범위는 HWPX `CHAR` 직렬화·재파싱 계약이며 #6875 및 07939의 외부 한컴
쪽수 주장을 닫거나 확정하지 않는다. 통합 PR 생성·merge 전에는 최신 integration head의 CI와
작업지시자 승인을 다시 확인한다.
