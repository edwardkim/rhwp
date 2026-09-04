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
| `.claude` | 25 |
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
