---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-23
---

# PR #7338 검토 — 표 설정 저장 동기화와 캡션 TABLE 속성 오염 방지

## 대상과 경로

- PR: [#7338](https://github.com/edwardkim/rhwp/pull/7338), 작성자 planet6897, base devel.
- 관련 이슈: [#7288](https://github.com/edwardkim/rhwp/issues/7288). 조판 본체는 별도이므로 닫지 않는다.
- source head: `5d30db9ca40b0b122a0465ff9d74bd4265982aa8`.
- 검토 base: `ed1ab115ad6278d005fa42a17c75db88716976dc` (#7348 병합).
- 로컬 통합 head: `164bddb089c4c8b38afdba3707fc3139f8383ed2`.
- 통합 tree: `2f15562ff63f64e3aae0ebddbefe9e2e6d0455ad`. merge-tree와 실제 로컬 merge 모두 충돌 없음.
- local branch: `review/pr7338-20260923`. 원 PR source를 수정하거나 push하지 않았다.
- 기본 경로: maintainer 일반. 보조: intake_and_review, local_validation. 기존 reviewer 요청 jangster77 유지.
- 제품 변경 1파일(+15/-2), 정식 회귀 검사 1파일(+201). renderer/typeset 수정 없음.

## 현재 devel과 중복 여부

PR 설명의 `raw_table_record_attr != 0`이면 저장기가 IR을 무시한다는 전제는 현재 source에도 맞지 않는다.
이미 `bd974e19b42e40e1207ed7de05fc33f3acd2f6c8` (#7323 수정, [#7326](https://github.com/edwardkim/rhwp/pull/7326)으로 병합)이
`serializer/control.rs::table_record_attr`에서 IR의 pageBreak/repeatHeader를 저장 비트에 반영한다.
따라서 현재 devel에서 PR의 6개 검사 중 5개는 수정 없이도 통과한다.

그러나 캡션 생성/제거가 `table.raw_table_record_attr = table.attr`로 개체 공통 속성을 TABLE 레코드에
덮어쓰는 결함은 남아 있다. 실제 입력의 `0x04000002`가 `0x282a2310`으로 오염된다.
저장기의 하위 비트 보정만으로 상위 비트 오염은 복원되지 않으므로 PR 전체를 중복으로 폐기할 수 없다.

PR은 캡션 경로의 잘못된 대입 2개를 제거한다. pageBreak/repeatHeader setter의 비트 동기화도
해당 비트만 마스킹하여 갱신하며 원본 상위 비트를 보존한다. 추가 메인터너 제품 보정은 하지 않았다.
PR 본문의 수정 전 4건 실패 주장은 현재 기준 결과와 구분해야 한다.

## 실제 경로와 계약

| 구현 주장 | 생산과 소비 경로 | 독립 기대값 / 검사 | 결과 |
| --- | --- | --- | --- |
| 설정값 저장 | set_table_properties_native → IR/raw bit 0–2 → serializer table_record_attr → HWP5 재파싱 | 요청한 0/1/2 및 bool이 왕복 후 동일 | 통합본 통과. 기본 저장 결함은 #7323에서 이미 수정 |
| 상위 비트 보존 | setter의 해당 비트 마스크 → 동일 serializer | fixture 원본 raw의 bit 3 이상 유지 | 통합본 통과 |
| 캡션과 TABLE 레코드 분리 | hasCaption 생성/제거 → common.attr만 변경 → TABLE raw 보존 → 저장/재파싱 | 원본 raw 전체 `0x04000002`, caption 존재 상태만 요청에 따라 변화 | base 실패 → 통합본 통과 |
| 기존 동작 보존 | HwpDocument setter 및 serialize_document → parse_document | #7323 기존 3개 정식 회귀 검사 | 3/3 통과 |

조판 좌표·분할·이어받기 알고리즘 및 기준값 변경은 비해당이다. 기존 문서의 캡션 모양·페이지 시각 일치나
WASM 브라우저 결과를 이번 검사로 주장하지 않는다. 저장 속성 보존만 검토하며 Visual Sweep은 수행하지 않았다.

## 검증 입력

실행 파일의 Git blob이 source/base/통합 head의 파일과 일치함을 확인했다. 신규 fixture나 임의 수정 없음.

| 저장소 입력 | SHA-256 |
| --- | --- |
| `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp` | `fbe318ac2fb20e42612571fe384567259b4785bc0375766c122cbc31cad828ca` |
| `samples/2010-01-06.hwp` | `d2562d9219fc1d491dd6b9f6d787314153246efb79e18c42c63830ac22194958` |

## 실행 결과

GitHub [Full CI 35718710078](https://github.com/edwardkim/rhwp/actions/runs/35718710078)의
head SHA가 source head와 정확히 일치하며 Build & Test, A–D, fmt/native·WASM·workspace Clippy가 성공했다.
CodeQL `35718710196`, adapter `35718710357`, proptest `35718710209`, CI Impact Policy도 성공했다.
광범위 전체 회귀와 lint는 local_validation 4.3.0 및 통합 워크플로우 3.2.2에 따라 중복 실행하지 않았다.
로컬 통합에서 source와 비교한 변경 대상 command/serializer/회귀 파일은 동일하고 메인터너 보정이 없다.

집중 검사는 Cargo가 만든 실제 rhwp 라이브러리에 정식 test source를 rustc --test로 링크해 실행했다.
이는 전체 Cargo/nextest 실행이 아니며 아래 범위에 한정한다.

```bash
cargo build --locked --lib --target-dir target/pr-review
rustc --edition 2021 --test output/7280/pr7338-roundtrip-review.rs \
  --extern rhwp=target/pr-review/debug/deps/librhwp.rlib \
  -L dependency=target/pr-review/debug/deps -o <base-or-integrated-test-binary>
<base-or-integrated-test-binary> --nocapture
CARGO_MANIFEST_DIR=/home/edward/mygithub/rhwp rustc --edition 2021 --test \
  tests/cases/issue_7323_table_page_break_save.rs \
  --extern rhwp=target/pr-review/debug/deps/librhwp.rlib \
  -L dependency=target/pr-review/debug/deps -o output/7280/pr7338-existing-7323-tests
output/7280/pr7338-existing-7323-tests --nocapture
```

- devel build 1m01s, 통합 build 17.88s, 모두 성공.
- 원 PR 정식 6개 검사 + reviewer의 캡션 on/off·전체 raw 보존 1개: base 5 PASS / 2 FAIL → 통합본 7 PASS / 0 FAIL.
- #7323 기존 정식 검사: 통합본 3 PASS / 0 FAIL.
- `git diff --check upstream/devel...HEAD`: 통과.
- 로컬 증적: `output/7280/pr7338-{base,integrated}-build.log`,
  `pr7338-{base,integrated}-tests.log`, `pr7338-existing-7323-tests.log`,
  `pr7338-roundtrip-review.rs`. reviewer 추가 검사는 진단 증거이고 원 PR의 정식 검사는 tests/cases에 존재한다.

## 최종 판정

**승인.** 리팩토링 구조에 맞춘 소스 재작업 없이 원 PR을 수용할 수 있다.
작업지시자의 병합 승인 후 원 head가 변경 없이 OPEN / MERGEABLE / CLEAN이고 필수 검사가 성공한 것을
재확인하여 2026-09-23 05:13:42 KST에 병합했다.
merge SHA: `c1ac0f987229956c839eb54f824741ff4ba43420`. 로컬 devel도 upstream/devel로 fast-forward했다.
#7288은 OPEN 상태를 확인했다. 이 검토 기록은 merge 이후 운영 기록으로 archive에 보존한다.
PR 설명의 기존 저장 문제와 이번 캡션 오염 해결 범위를 후속 결과 댓글에서 명확히 정정한다.

## Merge 후 contributor PR comment 계획

실제 merge SHA와 위 CI 링크를 연결하고, 감사와 함께 10개 집중 검사 통과 및 캡션 음성 대조 결과를 적는다.
#7323과 중복되는 기본 저장 개선, #7338 고유 캡션 수정, #7288 본체를 닫지 않는 범위를 구분한다.
시각 일치를 주장하지 않으므로 시각 asset은 추가하지 않는다. UTF-8 body-file로 게시 후 API로 본문을 확인한다.
일반 maintainer 후속 절차에서 해당 운영 기록만 반영한다. archive 이동은 완료했다.
