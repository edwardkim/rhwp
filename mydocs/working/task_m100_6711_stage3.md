---
kind: report
status: active
canonical: mydocs/plans/task_m100_6711.md
issue: 6711
last_verified: 2026-09-04
---

# #6711 Stage 3 — working 월별 archive 이동

## 1. 기준선과 분할 결정

Stage 2-B와 후속 comment 계획이 병합된 최신
`upstream/devel@2394c9044cfd1d3843fa8142e74858ec1bc8d78a`에서 dry-run했다. 시간 경계는
`Asia/Seoul`의 `2026-09-01T00:00:00+09:00`이다.

| 항목 | 수 |
| --- | ---: |
| 이동 전 `working` root Markdown | 2,289 |
| cutoff 이전 후보 | 2,239 |
| 9월 생성 유지 | 50 |
| Git history 누락 | 0 |
| `--diff-filter=A --follow` fallback | 3 |
| 기존 archive 동명 충돌 | 8 |

fallback 3건은 `--diff-filter=A --follow`에서 추가 commit을 찾지 못해 전체 이력에서 해당 경로가
처음 존재한 commit을 사용했다.

| 경로 | 최초 존재 commit | author 시각 |
| --- | --- | --- |
| `task_m100_3695_stage3.md` | `4c0a511fde84c8d762cccb715a149f7dac2667b2` | `2026-08-02T03:49:29+09:00` |
| `task_m100_3695_stage4.md` | `5410a9626030ab3531773cee8af45ec273f622e9` | `2026-08-02T04:00:53+09:00` |
| `task_m100_6395_stage2.md` | `b760dab95aef2d124bd8c5954eca48f17b301019` | `2026-08-30T17:28:05+09:00` |

2,239개를 한 번에 이동하면 rename을 전혀 인정하지 않는 보수적 경로 수가 최소 4,478개라 GitHub
PR files API의 3,000개 한도를 넘는다. basename 정렬을 기준으로 1,119개와 1,120개로 나눴으며,
각 batch의 외부 incoming-link 수정까지 dry-run에 포함했다.

| batch | 정렬 범위 | 후보 | 동일본 제거 | 외부 link source | 보수적 경로 수 |
| --- | --- | ---: | ---: | ---: | ---: |
| A | `agent_bug_hunter.md`–`task_m100_258_stage18.md` | 1,119 | 8 | 63 | 2,293 |
| B | `task_m100_258_stage19.md`–`typeset_stored_ladder_spacing_5801.md` | 1,120 | 0 | 103 | 2,343 예상 |

Batch B 수치는 Batch A 전 기준선의 dry-run 값이다. Batch A가 병합된 최신 `devel`에서 다시 측정한
값을 다음 절편의 제출 기준으로 사용한다.

## 2. Batch A 실행 결과

목적지가 없던 1,111개는 `mydocs/working/archives/`로 이동했다. 동명 archive가 있던 8개는
SHA-256이 같은 것을 확인한 뒤 기존 archive를 유지하고 root의 byte-identical 중복만 제거했다.
내용이 다른 동명 충돌은 0개다.

| 제거한 root 중복 | SHA-256 |
| --- | --- |
| `task_m100_1363_stage1.md` | `ac63005caa48ad44a780496a4ec852e3c76028088b2418934550c88d51e4d1ea` |
| `task_m100_1363_stage2.md` | `0802debd5e85c0305446b053b6f9ccbbd9e3b39423bdd4104402ddb165194f73` |
| `task_m100_1363_stage3.md` | `b1e0a5c3212845538cf990444d3f2f61c4d17c7f22a425c2f8854f84308ce612` |
| `task_m100_1363_stage4.md` | `fc0d214de9a098b7fa69dbe7a6fc7bd631712b87660291f6d37f3d1010f39181` |
| `task_m100_1363_stage5.md` | `5b23a03e2abd9f44b8fe0571d8417ea2ee68e945fab4a7dfb2fb45c0a83cf346` |
| `task_m100_1363_v2_stage1.md` | `267ebb54d2387237e03b707503048c0e16931f8dedf7322fc1b6120ca56fc01a` |
| `task_m100_1363_v2_stage2.md` | `71fd93952903b9b5c49c08d80d220fd7e3badf140ac4291750e1348c70d91958` |
| `task_m100_1363_v2_stage3.md` | `923f630100bf051978aad40471cc22a100f40ddecb4f7fdffb4ec806327f7947` |

이동 문서 내부 링크와 repository 전체 incoming link를 이전 source에서 해석한 논리 target 기준으로
재계산했다. 링크 내용이 바뀐 source는 93개이며, 이동 집합 밖 63개는 다음 범위다.

| 외부 범위 | 문서 수 |
| --- | ---: |
| `.agents` | 2 |
| `.agents` | 25 |
| `gym` | 30 |
| 다른 `mydocs` 역할 문서 | 6 |

Batch A 이동 직후 root에는 Batch B 후보 1,120개와 9월 문서 50개가 남았다. 이 보고서를 추가한
제출 tree의 root는 1,171개이며, 보고서는 9월 작업 문서이므로 archive 대상이 아니다.

## 3. 링크·metadata·범위 검증

기준선 Markdown은 `git cat-file --batch`로 읽고 현재 index와 비교했다. 이동 source와 target을
새 archive 경로로 정규화하고, 제거한 동일본 root 8개는 기준선 중복 source에서 제외했다.

| 검사 | 기준선 | Batch A | 차이 |
| --- | ---: | ---: | ---: |
| 저장소 내부 Markdown 링크 | 9,187 | 9,187 | 0 |
| 유효 링크 | 8,620 | 8,620 | 0 |
| historical broken link | 567 | 567 | 0 |
| 신규 / 소실 broken link | — | — | 0 / 0 |

`python3 scripts/check_markdown_links.py`는 609개 canonical 문서에서 오류 0건이다.
`--changed-from upstream/devel --forbid-redirect-references`가 표시한 12건은 이동 전에도 같은 논리
target이 없던 historical 오류이며 전수 multiset에서 신규·소실 0건으로 확인했다.

`python3 scripts/check_document_metadata.py`의 16건은 변경하지 않은 기존 4개 문서의 동일 오류다.
비 Markdown, Rust source·test, Cargo, WASM, workflow 변경은 0개다. `git diff --check`도 통과했다.

보고서를 포함한 최종 예상 변경 규모는 rename-aware 1,183개, rename을 전혀 인정하지 않는 보수적
경로 2,294개로 PR API 한도 아래다.

## 4. 다음 절차

1. 보고서를 포함한 staged tree의 rename·보수적 경로 수와 root 잔여 집합을 다시 확인한다.
2. 링크·metadata·diff 검사를 최종 재실행한다.
3. Batch A 결과 승인 뒤 한 commit으로 고정한다.
4. 별도 승인 뒤 push와 PR을 수행하고 exact-head CI를 확인한다.
5. merge 뒤 최신 `upstream/devel`에서 Batch B 1,120개와 incoming link를 다시 dry-run한다.
6. Batch B merge 뒤 Stage 4 전수 감사와 최종 보고로 #6711 종료 요건을 판단한다.

## 5. PR #6726 첫 head 회귀와 정정

첫 head `f73a68590d`의
[`Gym benchmark contracts`](https://github.com/edwardkim/rhwp/actions/runs/33865831824)는
2,125건 중 failures 5건, errors 16건으로 실패했다. 원인은 archive로 이동된 작업 기록을 Python
계약 테스트가 여전히 `mydocs/working/gym_*.md`에서 열던 하드코딩 경로였다. Markdown 링크 그래프는
통과했지만 비-Markdown 실행 소비자를 검사하지 않아 누락된 것이다.

수정은 root redirect stub을 되살리지 않고 다음 원칙으로 수행했다.

- 실제 파일을 읽는 Python·Rust 계약 테스트는 `mydocs/working/archives/`를 사용한다.
- 과거 이슈의 생성기와 현재 경로를 싣는 fixture·source 설명도 archive 위치에 맞췄다.
- 이동 문서 본문의 historical `canonical`과 과거 실행 증적은 내용 보존을 위해 바꾸지 않았다.
- 월별 archive 정본에 비-Markdown 경로 소비자 검색과 변경 종류별 추가 검증을 명시했다.

### 정정 검증

| 검증 | 결과 |
| --- | --- |
| CI와 같은 Gym 계약 명령 | 2,125 passed, 1 skipped |
| 변경된 Python·generator 관련 모듈 | 40개 모듈 통과 |
| `tools.agent_onboarding.test_rhwp_doctor` | 136 passed |
| 변경된 Rust `working` 계약 | 14개 generated suite에서 16 passed |
| native Clippy `-D warnings` | 통과 |
| WASM library Clippy `-D warnings` | 통과 |
| workspace build | 통과 |
| workspace all-targets Clippy `-D warnings` | 통과 |
| integration suite manifest | 1,152 sources, 48/48 targets 정합 |

변경된 agent 계약군에서 별도로 발견한 `test_agent_knowledge_map`, `test_agent_mcp_session`,
`test_agent_surface` 실패는 분리한 `devel@2394c9044c` worktree에서도 동일하게 재현되어 이번 변경의
신규 회귀에서 제외했다.

정정 뒤 누적 변경은 rename-aware 1,257개, rename을 전혀 인정하지 않는 보수적 경로 2,368개다.
PR files API 3,000개 한도 안이며, canonical Markdown 링크 오류는 계속 0건이고 metadata 오류는
기존 4개 문서의 16건 그대로다.

첫 정정 head `03417efe2d`의
[`Archive C 실제 실행`](https://github.com/edwardkim/rhwp/actions/runs/33867686045/job/101007764193)은
`agent_surface_skill_contract::working_doc_closes_issue_5326`에서 실패했다. 이 계약은 경로를
`.join("mydocs").join("working").join("agent_surface_skill.md")`로 나눠 조립했기 때문에 완성된
경로 문자열을 중심으로 한 첫 감사에서 빠졌다. `working` 다음에 `archives`를 추가하고, 저장소
전체에서 완성 문자열과 분할 조립 경로를 별도로 재검색했다.

해당 focused nextest는 1 passed이며, 수정 뒤 `cargo fmt --all -- --check`, native·WASM Clippy,
workspace build, workspace all-targets Clippy와 integration manifest 48/48을 다시 통과했다. 첫 head와
첫 정정 head는 모두 최종 후보가 아니며, 이 두 CI 실패를 보존한 채 다음 exact-head CI가 전부
성공해야 self-review로 넘어간다.

## 6. Batch B 최신 기준선과 dry-run

Batch A가 [PR #6726](https://github.com/edwardkim/rhwp/pull/6726)의 merge commit
`9e8e8bc567cb27b406a945a39637869c3b7fd3b7`으로 반영된 최신 `upstream/devel`에서 다시
계측했다. Git 최초 도입 시각을 직접 하위 Markdown 1,171개에 다시 적용한 결과는 Stage 3 최초
분할안과 일치했다.

| 항목 | 수 |
| --- | ---: |
| 이동 전 `working` root Markdown | 1,171 |
| cutoff 이전 후보 | 1,120 |
| 9월 생성 유지 | 51 |
| Git history 판정 불가 | 0 |
| 전체 이력 fallback | 3 |
| 기존 archive 동명 충돌 | 0 |

후보 범위는 basename 정렬 기준 `task_m100_258_stage19.md`부터
`typeset_stored_ladder_spacing_5801.md`까지다. fallback 3건은 1절에 기록한 기존 세 문서와
동일하다. 후보 path, NUL, 이동 전 파일 SHA-256 digest를 순서대로 누적한 inventory SHA-256은
`a060090a259dd53c89f94a9b6cf6a2159e626c018e0b7d823f03eb7e07e4746d`다.

이동 전 저장소 전수 Markdown 기준선은 13,186개 문서, 내부 링크 9,224개, 유효 8,671개,
historical broken 553개다. 후보 밖 incoming link source는 103개였다. 새 source 위치와 새 target
위치를 메모리상에서 먼저 계산해 전체 9,224개 링크의 논리 target multiset이 동일함을 확인한
뒤 실제 이동을 적용했다.

## 7. Batch B 실행 결과와 경로 소비자

목적지 충돌 없이 1,120개를 `mydocs/working/archives/`로 이동했다. 동일본 중복 제거와 divergent
충돌 보존은 모두 0건이다. 이동 문서의 상대 링크와 후보 밖 incoming link source 103개는 이동
전 논리 target을 유지하도록 새 상대 경로로 계산했다.

완성 경로 문자열과 분할 조립 경로를 별도로 감사한 결과, 다음 활성 font evidence 소비자는
이동 후 경로를 실제 입력 또는 생성 결과로 사용하므로 함께 정정했다.

- `scripts/font_rule_ledger.mjs`: 이후 생성되는 gate evidence 경로
- `scripts/font_typesetting_risk_evidence.mjs`: 이후 생성되는 evidence anchor 경로
- `mydocs/tech/investigations/issue-4962/font_typesetting_risk_contract.json`: 해시로 고정된 evidence
  입력 3개 경로

`task_m100_4741_stage5_validation.md`는 archive 이동에 따라 문서 내부 상대 링크 3개가
`../`에서 `../../`로 바뀌었다. 따라서 활성 contract의 해당 artifact SHA-256을 이동 전
`55779150ac67386b990ec456dfff182d4988b780a865cfe6f5d3d94d42566d06`에서 이동 후
`f003229f29f084a224d82fe79c6d2846e15831e417cedc42383015229db0282d`로 갱신했다. 나머지
evidence 입력은 내용 hash가 동일하다. 과거 실행 결과 JSON, CI classifier fixture, source 주석,
이동 문서의 historical `canonical`은 현재 파일을 여는 실행 경로가 아니므로 원문을 보존했다.

보고서를 포함한 제출 후보는 rename-aware 1,227개다. rename 판정에 기대지 않는 보수적 경로
수는 이동 old/new 2,240개와 수정 107개를 합한 2,347개로 GitHub PR files API 3,000개 제한
아래다. 변경 범위는 `mydocs` 1,223개, 이동 문서로 들어오는 Markdown link source인 `pdf` 1개와
`samples` 1개, 활성 소비자 `scripts` 2개다. Rust source·test, Cargo, WASM, workflow 변경은 없다.

## 8. Batch B 검증

| 검사 | 결과 |
| --- | --- |
| `git diff --check` | 통과 |
| canonical `check_markdown_links.py` | 609개 문서, 오류 0건 |
| 전수 Markdown link multiset | 9,224개 유지 |
| 전수 유효 / historical broken | 8,671 / 553, 기준선과 동일 |
| cutoff 이전 direct Markdown 잔여 | 0개 |
| `check_document_metadata.py` | 변경하지 않은 기존 4개 문서의 16건만 재현 |
| font ledger·risk Node 계약 | 24 passed |
| risk contract evidence input | 6개 path·SHA-256 일치 |
| 범위 감사 | Rust·Cargo·WASM·workflow 변경 0건 |

## 9. 남은 절차

1. 보고서를 포함한 staged tree의 변경 수·범위와 링크·metadata 결과를 최종 재검사한다.
2. 메인테이너가 Batch B 결과를 승인하면 한 commit으로 고정한다.
3. 별도 승인 뒤 원격 push와 PR을 생성하고 exact-head CI·self-review·정상 merge commit 절차를
   수행한다.
4. Batch B merge 뒤 최신 `upstream/devel`에서 Stage 4 전수 감사와 최종 보고로 #6711 종료
   요건을 판단한다.
