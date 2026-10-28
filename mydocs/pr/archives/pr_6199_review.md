---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6199 review - #6172 합성 네모 숫자 전진폭

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6199
- 작성자: `planet6897`
- 원 PR head: `7e52aa3d402f`
- 통합 검토 브랜치: `review/planet6897-6199-6217-20260827`
- 최신 기준: `upstream/devel@9d6f69b4d1a0`
- 검증 실행 기준: `upstream/devel@584320e0ee02`
- 원 PR 상태: non-draft, source CI green, comments/reviews 0건
- 관련 이슈: #6172

## 검토 판단

**수용 권고**. 렌더러가 합성으로 그리는 PUA boxed number를 측정 단계에서 같은 시각 폭을 가진
`□`로 재측정해, `② 입항회수` 행의 `200`과 뒤따르는 `□-□□□`가 서로 겹치지 않게 한다. 변경 범위는
`char_width_decision`의 측정 입력에 한정되어 있고, 실제 텍스트 값이나 word spacing 경로에는 영향을
주지 않는다.

## 증적과 검증

- 원 PR 시각 보고서: `mydocs/report/pua-boxed-advance-6172/{before_p1,after_p1,oracle_p1}.png`
- 검토자가 직접 확인한 대표 after/oracle: 합성 상자와 실제 `□`가 일정 간격으로 배치되며 겹침 없음
- 파일 버전 증적: `mydocs/pr/assets/pr_6199_6217_2599643_port_call_form_hwp_info.json`
- focused test: `issue_6172_pua_boxed_number_advance` 1 pass
- 공통 검증: fmt, suite manifest, unit tier, clippy, 전체 nextest, Native Skia 3종, WASM build 통과.
  상세 명령과 숫자는 통합 구현 문서에 기록했다.
- 2026-08-28 최신 `upstream/devel@9d6f69b4d1a0`로 충돌 없이 rebase했다. 사용자 지시에 따라 별도
  중복 테스트는 수행하지 않았다.

## 후속

통합 PR에는 #6172의 사용자-visible 겹침이 재현 fixture와 회귀 테스트로 고정됐음을 기록한다.
