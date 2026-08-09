---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4171 검토 - CFB 루트 CLSID 보존과 HWP3 상대 크기 기본값 보정

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 통합 PR / 작성자 | [#4171](https://github.com/edwardkim/rhwp/pull/4171) / @jangster77 |
| 기준 `devel` | `9dbd3dc6c49c36e8d1012a19ec60dea1abd5123c` |
| 통합 code head | `c4f85c0a0aadc983957f0d44f5375c2b5744baed` |
| 가시성 검토 브랜치 | `review/johndoekim-20260807-integration` |
| 원 PR | [#4144](https://github.com/edwardkim/rhwp/pull/4144), [#4160](https://github.com/edwardkim/rhwp/pull/4160) / @johndoekim |
| 원 contributor head | #4144 `52ecd911e8676e13ec0b4fd5c949c497c5523915`, #4160 `08d5e4f4250a2406cb990303c22d31d7f7348d6f` |
| 원 PR 적용 순서 | #4144 → #4160 |
| 연동 이슈 | `Closes #4097`, `Closes #4141` |

라우팅은 `maintainer_general`을 기본으로 하고, `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `multi_pr_update_branch`, `review_only_fast_pass`를 보조 경로로 적용했다.
원 contributor commit은 rebase, amend, reset, force-push하지 않았다. 최신 `devel` 위 검토 브랜치에
두 원 PR을 의존 순서대로 cherry-pick하고, 아래 메인터너 보정만 추가했다.

- #4144는 mini CFB 재포장과 HWP3 OLE 서브 스토리지 승격 시 OLE 루트 CLSID를 보존한다.
  보존하지 않으면 한컴이 OLE 개체를 식별하지 못해 틀과 선택 핸들만 보이고 내용이 비는 문제가 생긴다.
- #4160은 HWP3에서 HWP5/HWPX로 저장할 때 `CharShape.relative_sizes`의 미지정 기본값을 OWPML
  기본값인 `100`으로 적용한다. 기존 `0`은 한컴에서 0.12pt 수준의 본문으로 해석될 수 있었다.

이 통합 PR은 parser, serializer, model과 HWP3/HWPX 저장 계약을 바꾼다. rhwp의 layout/paint 또는
PDF export 구현 자체는 바꾸지 않는다. #4144에 포함된 한컴 PDF 세 건은 실제 PDF이며 모두 A4 46쪽으로
확인했다. #4160의 상대 크기 변경은 저장 포맷 계약 테스트로 검증했으며, 남아 있는 별도 fidelity 과제는
이 기본값 보정의 수용 범위를 넓히지 않는다.

## 발견한 문제와 메인터너 보정

통합 검토에서 `root_clsid`가 CFB v4의 첫 디렉터리 섹터 오프셋을 `(SID + 1) * sector_size`로 계산하는
문제를 발견했다. CFB v3와 v4는 모두 파일 앞 512바이트가 헤더이므로 올바른 위치는
`512 + SID * sector_size`다. v3의 섹터 크기는 512바이트여서 두 식이 우연히 같지만, v4의 4096바이트
섹터에서는 다른 위치를 읽는다.

메인터너 보정 `c4f85c0a0`은 `src/parser/cfb_reader.rs`의 계산을 고치고, SID 1의 CFB v4 디렉터리에서
CLSID를 읽는 회귀 테스트를 추가했다. CFB v3 동작, `mini_cfb`의 v3 writer 계약, contributor의 CLSID
보존 변경은 바꾸지 않았다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 적용 source SHA 재확인 | #4144와 #4160의 원격 head가 누적 검토에 사용한 SHA와 일치 |
| 병합 가능성 | `upstream/devel`이 통합 branch의 조상이며 10개 commit이 선형으로 적용됨 |
| diff 정합 | `git diff --check upstream/devel...HEAD` 통과 |
| CFB v4 보정 | `cargo test --profile release-test cfb_reader::tests::root_clsid_reads_v4_directory_after_fixed_header --lib` 1 passed |
| #4097 계약 | `cargo test --profile release-test --test issue_4097_mini_cfb_root_clsid` 3 passed |
| #4141 계약 | `cargo test --profile release-test --test issue_4141_hwp3_relative_size_contract` 5 passed |
| Rust 전체 통합 | `cargo test --profile release-test --tests` 통과 |
| 정적 검사 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과 |
| WASM 호환성 | `cargo check --target wasm32-unknown-unknown --lib` 통과 |
| GitHub code candidate | code head `c4f85c0a0`의 CI, CodeQL, Render Diff가 모두 성공 |

## 수용 판단과 merge 조건

**메인터너 보정 포함 수용 권고.** 두 PR의 저장 형식 계약과 CFB v4 경계 조건을 모두 검증했고, 전체
release-test와 WASM 대상도 통과했다.

이 문서와 오늘할일은 통과한 code head 뒤에 붙이는 trailing documentation commit이다. 문서 push 뒤에는
최신 PR head의 preflight와 branch-protection aggregate가 성공하고, `mergeable=MERGEABLE`,
`mergeStateStatus=CLEAN`인지 다시 확인해야 한다. 그 조건과 작업지시자의 merge 승인이 충족되면 #4171을
병합하고 #4097·#4141의 자동 종료와 원 PR #4144·#4160의 후속 정리를 확인한다.
