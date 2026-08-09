---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4165 검토 - 한글 버전 오라클 r1 정합 검증

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4165](https://github.com/edwardkim/rhwp/pull/4165) |
| 작성자 | `kevin9327` |
| 대상 | `devel` |
| contributor 원 source head | `39c535583cb5e18bc4b54e79f825d73a0ac8a265` |
| 보정 뒤 code candidate | `ae63f789860ca1af3131d6d644cfd5dc1fda7055` |
| 가시성 검토 브랜치 | `review/kevin9327-4165-20260808` |
| local 검토 기준 `devel` | `d9c530ee8ed4bd0830ff35bc47e552bb0f32274f` |
| 원 PR 표시 규모 | 13 files, +2,230 / -0 |
| 최신 기준 실제 diff | 6 files, +610 / -3 |
| 문서 작성 시점 원격 상태 | `mergeable=CONFLICTING`, `mergeStateStatus=DIRTY`; 보정 push 뒤 재확인 필요 |

라우팅은 `collaborator_external_pr`을 기본으로 하고 `intake_and_review`, `local_validation`,
`rework_and_exceptions`를 적용했다. GitHub에 보이는 1,000줄 초과 규모는 이미 `devel`에 병합된
#4136의 누적 commit과 세 번의 base merge를 포함한다. 최신 기준의 고유 변경은 보고서·Python 검증기·
Markdown 링크 검사뿐이며 renderer, Rust, fixture, 기준 PDF는 바꾸지 않으므로 시각 fixture 증적 경로는
적용하지 않았다.

## 기여자 변경 검토

기여자 변경은 r1 보고서의 행 단위 데이터에서 41개 정합 조건을 재계산하는
`verify_r1_report_consistency.py`와 그 결과 보고서를 추가한다. 또한 보고서의 출처별 `prism_downloads`
건수를 141에서 138로 고치고, 파일명 안의 백틱 렌더와 PowerShell BOM 계약을 보정한다.

검증기는 한글 설치나 10k 코퍼스 없이 보고서 자체의 121 PAGE_DELTA, 123 BREAK_DIFF, 3 PARA_DIFF,
7 미확정, 5 2020↔2022 행을 재계산한다. 따라서 오라클 측정을 재실행하는 도구가 아니라 저장소에
보존된 결과의 내부 정합을 지속적으로 확인하는 도구다.

## 발견한 문제와 메인터너 보정

기여자 원 commit은 재작성하지 않았다. 아래 변경은 contributor head 뒤의 별도 commit으로만 추가했다.

1. `303c550e3`은 Markdown 링크 검사에서 인라인 코드 영역을 마스킹한다. 파일명 속 `[별지 제3호]`를
   내부 링크로 오인하던 false positive를 막고 회귀 테스트를 추가했다.
2. contributor source와 최신 `devel`의 충돌은 `mydocs/orders/20260807.md` 한 파일뿐이었다. PR 고유
   변경이 없는 과거 오늘할일이므로 `7470966f9`에서 최신 `devel` 내용을 보존해 conflict를 해소했다.
3. `9cbb8dd35`은 최신 r1 보고서의 §4 다섯 열(재실행 동일 2, 재실행 실패 4, 최초 실행 ERR 1)을
   검증기가 해석하도록 보정했다. Markdown 백틱·이중 백틱·`<code>` 경로를 하나의 문서 경로로
   처리하고 HTML entity를 복원하며, 최초 실행 ERR을 재실행 실패와 분리해 §6 행과 대조한다.
4. `ae63f7898`은 정합성 보고서의 과거 `39 PASS · 2 CONTRADICTION` 결론을 현재 보고서와 실제
   실행 결과인 `41 PASS · 0 FAIL · 0 CONTRADICTION`으로 갱신했다. 이는 원자료 재측정 결과가 아니라
   보고서 내부 데이터의 최신 정합 결과임을 명시한다.

## 수행한 검증

| 검증 | 결과 |
| --- | --- |
| 최신 `devel` merge tree | `git merge-tree --write-tree upstream/devel HEAD`가 충돌 없이 통과했고, `upstream/devel`은 candidate의 조상이다 |
| 오라클 파서 단위 테스트 | `python3 tools/hangul_version_oracle/test_verify_r1_report_consistency.py`에서 3 passed |
| r1 정합 전수 검사 | `python3 tools/hangul_version_oracle/verify_r1_report_consistency.py`에서 41 checks: 41 PASS, exit 0 |
| TSV 출력 경로 | `--tsv` 실행이 헤더 포함 42행을 생성했다 |
| 링크 검사 단위 회귀 | `python3 scripts/tests/test_check_markdown_links.py`에서 5 passed |
| scripts Python 회귀 | `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`에서 118 passed |
| Python 구문 | `python3 -m py_compile`로 변경 Python 파일을 검사해 통과 |
| 변경 문서 링크 | `python3 scripts/check_markdown_links.py --changed-from upstream/devel`에서 516개 문서, 내부 상대 링크 이상 없음 |
| diff/LFS 사전 판독 | `git diff --check` 통과, contributor head부터 candidate까지 LFS 추적 파일 없음 |

최신 기준 실제 diff에는 Rust, renderer, HWP/HWPX fixture, PowerShell 변경이 없다. 따라서 Cargo 전체,
Native Skia, Windows COM 재실행은 이번 Python·문서 보정의 검증 범위를 벗어나며 실행하지 않았다.

## 수용 판단과 merge 조건

**메인터너 보정 포함 수용 후보**다. r1 보고서·가이드·하니스 계약의 41개 내부 정합 검사가 모두 통과했고,
링크 검사는 파일명 속 대괄호를 링크로 오인하지 않는다.

보정 commit과 최신 `devel` 병합이 code/test 변경을 포함하므로 review-only fast-pass 대상이 아니다.
작업지시자의 push 승인 뒤 contributor source branch에 현재 candidate를 반영하고, 최신 head의 Full CI와
CodeQL이 모두 성공하며 mergeable 상태를 재확인한 뒤 작업지시자의 merge 승인에 따라 병합한다.
