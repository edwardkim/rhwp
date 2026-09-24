---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7335 검토 기록 — PR 본문 Visual Sweep 증적 표시

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7335](https://github.com/edwardkim/rhwp/pull/7335) |
| 작성자·reviewer | `jangster77` collaborator self-review |
| base / code candidate | `cb1d2399c8993b4d4f5c14407d793c99eebe7270` / `dfe4f5ecc70e57c6d850a0f75ac3955458607538` |
| code candidate CI | [CI #35697908106](https://github.com/edwardkim/rhwp/actions/runs/35697908106) 성공 |
| PR 상태 기록 시점 | Open, `MERGEABLE`, `CLEAN` |
| 변경 범위 | PR 템플릿·agent 지침·기여자 안내·Visual Sweep/PR review 정본 문서 |

## 변경과 계약

기존 규칙은 렌더링 변경에서 Visual Sweep을 실행하고 대표 PNG를 보존하도록 요구했지만, PR 본문에는
경로·수치·review 문서 링크만 남아 reviewer가 결과를 직접 보기 어려웠다. 실제 [#7334](https://github.com/edwardkim/rhwp/pull/7334)도
Visual Sweep 설명은 있으나 본문에 PNG Markdown 이미지가 없었다.

이 PR은 renderer/layout/typeset/paint 또는 사용자-visible WASM 렌더링 변경에서 다음을 같은 계약으로
정한다.

1. 비교를 마친 최종 PR head에 공개 가능한 대표 review·standalone overlay PNG를 안정 경로로 보존한다.
2. PR 본문은 `headRepositoryOwner/headRepository`와 정확한 `headRefOid`의 raw URL을 Markdown image로
   표시한다. asset 경로·임시 output·review 문서 링크만으로 대신하지 않는다.
3. Native/fresh WASM 등 실제 실행한 출력 경로마다 review와 overlay를 표시하고, 미실행 경로는 사유를 남긴다.
4. 게시 뒤 PR body의 SHA·한글·URL, asset의 해당 head 존재, GitHub 화면의 실제 렌더링을 확인한다.
5. merge 뒤 contributor comment는 동일 asset을 `edwardkim/rhwp`의 실제 merge SHA raw URL로 다시 고정한다.

PR 번호가 아직 없어서 asset 경로를 미리 정할 수 없는 경우에는 `issue_<N>_<topic>/`처럼 issue 또는 변경
주제 기반의 안정 경로를 사용한다. 이 방식은 PNG 경로를 PR 번호로 바꾸기 위한 별도 trailing commit을 만들지
않는다.

## 파일별 검토

| 파일 | 판정 | 근거 |
| --- | --- | --- |
| `.github/pull_request_template.md` | 충족 | 조판 변경 근거 다음에 Native/fresh WASM review·overlay Markdown image 표와 head SHA 규약을 추가했다. |
| `AGENTS.md`, `CLAUDE.md` | 충족 | agent가 “결과보고 연결”에서 끝내지 않고 PR 본문에 실제 이미지를 표시하도록 공통 규칙을 맞췄다. |
| `CONTRIBUTING.md` | 충족 | 공개 가능한 대표 증적 보존과 개인정보·대형 원본·중간 산출물 제외를 구분해 기존 “이미지 커밋 금지”의 충돌을 해소했다. |
| `visual_sweep_guide.md` | 충족 | merge 전 PR head SHA 증적과 merge 뒤 merge SHA comment 증적을 별도 절차로 분리했다. |
| `visual_fixture_evidence.md`, `pr_review_workflow.md`, `intake_and_review.md` | 충족 | 작성자·메인터너·reviewer가 asset 존재, PR body, 실제 표시를 각각 확인하도록 연결했다. |

CI workflow나 실행 정책은 변경하지 않았다. 이 PR은 renderer·WASM 구현, fixture, 기준 PDF, baseline, PNG asset을
변경하지 않으므로 Visual Sweep 실행 자체는 비대상이다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| PR 본문 직접 증적 정본 anchor·템플릿 raw URL 계약 점검 | 통과 |
| [CI #35697908106](https://github.com/edwardkim/rhwp/actions/runs/35697908106) | Build & Test, lint, Native Skia 성공; [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35697908028)·[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35697908174)·[Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35697908192)도 성공 |
| Rust/WASM/Visual Sweep 실행 | 문서·PR 템플릿만 변경하므로 비대상 |

## 최종 판정

**승인.** code candidate `dfe4f5ecc`의 Full CI가 성공했고, PR 본문에 표시할 Visual Sweep 증적의
작성·보존·검토·병합 후 고정 규칙이 일관된다. 이 trailing commit에는 이 review 문서와 오늘할일만
추가한다. push 뒤 최신 trailing head의 review-only CI, `MERGEABLE`, `CLEAN`을 다시 확인한 뒤에만
병합 후보로 진행한다.
