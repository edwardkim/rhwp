---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4178 검토 - 누름틀 직렬화와 WebHwpCtrl P0 하니스

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4178](https://github.com/edwardkim/rhwp/pull/4178) |
| 작성자 | `planet6897` |
| 대상 | `devel` |
| contributor 원 code head | `72704d909265811bf3a643a4cb6bfbe05d9682b2` |
| 보정 뒤 code head | `7f38e41efea7c29c8e7c22ccf0299a9928db9b1d` |
| 가시성 검토 브랜치 | `review/planet6897-4178-20260808` |
| local 검토 기준 `devel` | `80fd91263132d6cdca0220c164e7d26586d5a3ea` |
| 원격 메인터너 보정 commit | `3cf9e53824ea26b630ce571c2da259d287e327bb`, `7f38e41efea7c29c8e7c22ccf0299a9928db9b1d` |
| 원 PR 규모 | 21 files, +17,623 / -5 |
| 문서 작성 시점 원 PR 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`; merge 전 재확인 필요 |

라우팅은 `collaborator_external_pr`을 기본으로 하고 `intake_and_review`, `local_validation`,
`review_only_fast_pass`, `rework_and_exceptions`를 적용했다. 1,000줄을 넘는 PR이므로 serializer 회귀와
P0 하니스 계약을 분리해 검토했다. renderer, typeset, paint, PDF 기준 자료는 변경하지 않으므로 시각 fixture
증적 경로는 적용하지 않았다.

## 기여자 변경 검토

`src/serializer/body_text.rs`는 같은 문자 위치의 FIELD_END, FIELD_BEGIN, 빈 필드 FIELD_END를 분리해
LIFO 파서와 맞는 순서로 직렬화한다. `field_begin_emission_order` 회귀 테스트는
`issue-986-receipt.hwp`에서 한 필드에 값 저장 뒤 인접 빈 필드 값과 전체 필드 수가 유지되는지 확인한다.
이 범위는 `edit fill-fields`와 native HWP 왕복 경계를 직접 다룬다.

나머지 변경은 WebHwpCtrl P0의 공식 문서 3종 기반 API/Action/ParameterSet 원장과 한글 COM 대조
하니스다. 아직 호환 API 구현을 추가하지 않으며, P0은 오라클 산출물·반환값(L2)·저장 문서 상태(L3)를
측정하는 범위로 한정된다.

## 발견한 문제와 메인터너 보정

기여자 commit은 재작성하지 않았다. 아래 보정은 contributor 원 head 뒤의 별도 commit으로만 추가했다.

1. `hwp.Version`을 `'12,'` 접두사로만 비교해 `12.0.0.4547` 형식의 한글 2022를 거부할 수 있었다.
   `3cf9e5382`에서 공통 major 파서가 comma/dotted 형식을 모두 12로 판정하도록 보정했다.
2. 버전 거부 또는 실패 뒤에도 이전 `<id>.returns.json`과 저장 HWP가 남고, `--skip-ocx`와
   `compare.py`가 이를 다시 읽을 수 있었다. 실행 전 해당 시나리오 산출물을 안전하게 제거하고, skip 시
   저장된 oracle version을 검증하며, 이번 실행에서 성공한 시나리오만 비교하도록 보정했다.
3. `taskkill /F /IM Hwp.exe` 및 `HwpFrame.exe`는 게이트와 무관하게 사용자가 열어 둔 모든 한글을
   종료했다. 기존 프로세스가 있으면 `OCCUPIED`로 중단하고, timeout 뒤 남은 PID는 기본적으로
   `LEFTOVER`로 실패시킨다. 전용 Windows 계정에서만 `--cleanup-spawned`를 명시해 새 PID 정리를
   선택할 수 있다.
4. 공식 문서가 선언한 ParameterSet Item 521개는 추출 검증에 없었다. set 수와 비어 있지 않음만 보던
   검사를 총 item 수 521까지 단언하도록 보강했다.
5. `HwpObject.Quit()` 직후 한글 프로세스가 비동기로 종료되어 첫 전체 gate가 일시적으로 `LEFTOVER`를
   보고했다. `7f38e41ef`에서 새 PID를 기본 10초 동안 재확인한 뒤에만 `LEFTOVER`로 판정하도록 보정하고
   계약 테스트를 추가했다. 사용자가 열어 둔 기존 한글 프로세스는 계속 종료하지 않는다.

## 수행한 검증

| 검증 | 결과 |
| --- | --- |
| 최신 `devel` merge tree | `git merge-tree --write-tree upstream/devel upstream/pr4178-head`가 충돌 없이 tree `a36c328c` 생성 |
| source 적용과 diff | 원 두 commit을 최신 `devel` 위에 순서대로 적용했고 `git diff --check` 통과 |
| serializer 회귀 | `CARGO_TARGET_DIR=target/review-planet6897-4178 CARGO_INCREMENTAL=0 cargo test --profile release-test --test field_begin_emission_order`에서 1 passed |
| P0 하니스 계약 | `python3 tools/hwpctrl_compat/test_harness_contract.py`에서 9 passed |
| Python/Node 정적 검사 | `python3 -m py_compile tools/hwpctrl_compat/*.py`, `node --check tools/hwpctrl_compat/runner_rhwp.mjs` 통과 |
| 원장 정합 | `python3 tools/hwpctrl_compat/build_ledger.py --check`에서 `0/484 완료`로 통과; 스펙 실측 API 122, Action 312, ParameterSet 50 / Item 521 |
| 패키지 경계 | `cd npm/hwpctrl-ocx && npm pack --dry-run --json` 통과, spec 4개와 package metadata만 포함 |
| 보정 code head GitHub 검증 | code head `7f38e41ef`의 [Full CI run 31204218519](https://github.com/edwardkim/rhwp/actions/runs/31204218519)와 [CodeQL run 31204217839](https://github.com/edwardkim/rhwp/actions/runs/31204217839)가 모두 성공 |
| Windows COM 단독 오라클 | Windows 10 RDP session 2에서 `field-read` 실행: 한글 `12, 0, 0, 535`, 호출 8건, `fatal=None`, call error 0건, 종료 뒤 Hwp/HwpFrame/python 잔류 없음 |
| Windows 전체 gate | 같은 대화형 session에서 `run_gate.py --impl legacy --only field-read --timeout 120`가 exit 0, `field-read=OK`, 비교 대상 1건으로 완료. L2는 8건 중 MATCH 3, VALUE_DIFF 2, MISSING_API 3이며 P0 측정 결과로 보존했고 L3 저장 검증 대상은 없음 |

Windows SSH Services session 0에서 직접 COM을 생성하면 `pyhwpx.Hwp()`가 초기화 단계에서 멈췄다.
해당 python/Hwp PID는 즉시 정리했다. 이후 활성 RDP session 2의 일회성 interactive scheduled task로
같은 러너를 실행했다. 첫 전체 gate에서 Quit 뒤의 비동기 종료 지연을 관찰해 `7f38e41ef`로 보정한 뒤
재실행했고 위의 exit 0 결과와 잔류 프로세스 없음까지 확인했다. 검증 task와 임시 파일은 결과 기록 뒤
제거했다. 따라서 실제 COM 검증은 비대화형 SSH가 아니라 한글이 동작하는 interactive session에서만
수행했다.

## 수용 판단과 merge 조건

**메인터너 보정 포함 수용 후보**다. serializer 회귀는 focused test로 통과했고, P0 하니스는 stale
정답지·오라클 버전·프로세스 종료 경계를 보정해 안전하게 실패하도록 바뀌었다. 보정 code head의 Full CI와
CodeQL도 모두 성공했다.

이 trailing review·오늘할일 commit은 `7f38e41ef` 뒤의 review-only 변경만 포함한다. 최신 head의
preflight와 Build & Test aggregate가 fast-pass 조건으로 성공하고, mergeable 상태를 다시 확인한 뒤
작업지시자가 merge를 승인할 때만 병합한다.
