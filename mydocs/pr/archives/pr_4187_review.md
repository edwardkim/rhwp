---
kind: pr_review
status: maintainer-correction-ready-for-push
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4187 검토 기록 - 웹한글컨트롤 호환 층 P1~P3

## 결론

**메인터너 보정 포함 수용 후보.** 원 contributor head `84b0aae11f33c220a0e172a6eddd120cc064f713`의
Full CI는 성공했지만, 이전 source 갱신 과정에서 빠진 패키지 소비자 계약과 macOS/Linux 게이트 보정이
없었다. collaborator 보정 `9ffc8009594315299349a1e53568e377038c5238`은 이를 원 head 뒤의 별도
commit으로 복원했고, Linux에서 계약 검사와 WASM self-check 44개 시나리오를 통과했다.

보정은 코드·테스트를 포함하므로 review-only fast-pass 대상이 아니다. 이 문서 시점의 최종 조건은
보정 head를 contributor source branch에 push한 뒤 최신 head의 Full CI, CodeQL, Render Diff 및
mergeable 상태가 모두 성공하고 작업지시자가 merge를 승인하는 것이다.

## 검토 경로와 기준

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
           rework_and_exceptions.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_external_pr.md, intake_and_review.md,
                  local_validation.md, rework_and_exceptions.md,
                  multi_pr_update_branch.md
```

| 항목 | 내용 |
| --- | --- |
| PR | [#4187](https://github.com/edwardkim/rhwp/pull/4187) |
| 작성자 | `planet6897` |
| 대상 / source | `devel` / `planet6897:pr/devel-hwpctrl-p1p3` |
| contributor source head | `84b0aae11f33c220a0e172a6eddd120cc064f713` |
| collaborator 보정 head | `9ffc8009594315299349a1e53568e377038c5238` |
| 문서 작성 시점 기준 devel | `828eabc19a4953a684e05d523a614256dae28b26` |
| 원 PR 규모 | 65 files, +22,585 / -742 |
| 문서 작성 시점 원격 상태 | `MERGEABLE`, `CLEAN`, `maintainerCanModify=true` |
| 관련 참조 | [#4178](https://github.com/edwardkim/rhwp/pull/4178) P0 차등 하니스 |

head SHA, CI, mergeability는 작성 시점 참고값이다. merge 직전 원격 head와 required check를 다시 확인한다.

## 원 변경과 메인터너 보정

- contributor 원 변경은 `@rhwp/hwpctrl` 호환 계층의 P1~P3 기능, Rust/WASM API, 44개 Oracle 시나리오와
  193/484 원장을 추가한다.
- source branch가 갱신되면서 이전 maintainer 보정이 history에서 사라졌다. 이 상태에서는 POSIX에서
  `python3` 대신 `python`을 고정 호출하고, Windows에서만 가능한 Hancom COM Oracle을 기본 경로로
  간주하며, package export 소비 계약을 검증하지 못한다.
- `9ffc80095`는 package ESM export/소비 계약, OS별 Python runner, Windows COM Oracle 및 macOS/Linux
  WASM self-check 분기, read-only fixture 비교, 비교 불일치의 비영(0) 종료 방지와 관련 개발 문서를
  한 commit으로 복원했다.
- 보정은 renderer, layout, paint, HWP/HWPX/PDF fixture, golden을 변경하지 않는다. 따라서 PDF 또는
  브라우저 시각 증적은 이 보정의 판단 근거가 아니다.

## 완료한 검증

모든 명령은 `review/planet6897-20260809-r2`에서 collaborator 보정 head 기준으로 완료했다.

| 검증 | 결과 |
| --- | --- |
| `npm --prefix npm/hwpctrl-ocx run test:contract` | 통과, Node package 소비 계약 1건 |
| `node tools/hwpctrl_compat/python_runner.mjs test_harness_contract.py` | 통과, Python harness 계약 12건 |
| `npm --prefix npm/hwpctrl-ocx run ledger:check` | 통과, 원장 `193/484` |
| `wasm-pack build --target web --out-dir pkg` | 새 WASM 산출물을 생성했다. 산출물 소비 전 wrapper가 종료 상태를 반환하지 않아, 아래 gate 성공 후 해당 wrapper만 종료했다. 이 명령의 종료 상태를 PASS로 기록하지 않는다. |
| `npm --prefix npm/hwpctrl-ocx run gate` | 통과, Linux `wasm-self-check`, 44 scenarios, `comparisonStatus=NOT_RUN`, non-WASM 오류 없음 |
| `git diff --check 84b0aae..9ffc800` | 통과 |
| 최신 `upstream/devel` merge tree 및 `git diff --check` | 충돌 없이 통과, merge tree `903239972f9ae0f726beacc1327186ee880c40dc` |

원 contributor head `84b0aae`의 GitHub Full CI, CodeQL, Render Diff는 성공했다. 다만 이 보정에는
code/test 변경이 있으므로 그 성공 결과를 보정 head에 재사용하지 않는다.

## 문서 범위

이 branch에는 `mydocs/orders/20260809.md`를 추가하지 않는다. source branch에는 해당 파일이 없고,
최신 `devel`에는 다른 PR의 오늘 기록으로 이미 존재한다. source에 복사하면 add/add 충돌과 unrelated
오늘 기록 중복을 만들기 때문이다. merge tree에서 최신 `devel`의 오늘 기록이 보존되는 것을 확인했다.

## 위험과 후속 경계

- Linux/macOS self-check는 wasm 호환 계층의 호출·순서·오류·저장 계약을 검증한다. Hancom 2022 COM과의
  새 Oracle 수집은 Windows에서만 수행한다.
- P1~P3 범위 밖 API와 오라클 데이터의 품질 판정은 이 보정에서 확장하지 않는다.
- PR 본문에는 issue close 키워드가 없다. 따라서 #4187 병합 뒤 이 PR을 근거로 별도 issue를 close하지 않는다.
- 원 PR이 대형 변경이므로 보정 push 뒤의 최신 Full CI가 최종 안전망이다.

## 최종 권고

보정과 이 검토 기록을 같은 contributor source branch에 push하고 Full CI를 재실행한다. 최신 head의
필수 check가 성공한 뒤 작업지시자 승인으로 PR #4187을 merge한다.
