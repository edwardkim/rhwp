---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7067 — HWP3 문단 스타일 참조 검토

**최종 판정: 승인.** HWP3 문단의 스타일 참조 보존이라는 한정된 수정은 수용 가능하다.
#4680 전체의 한컴 개방 거부·텍스트 소실·쪽 넘침을 해결했다고 판단하지 않으며 해당 issue는 열린 상태로 유지한다.
이 검토 판정은 PR 병합을 뜻하지 않는다. review-only 최종 head의 CI를 별도로 확인한다.

## 대상과 head 정렬

| 항목 | 확인 내용 |
| --- | --- |
| PR / 작성자 | [#7067](https://github.com/edwardkim/rhwp/pull/7067) / planet6897 |
| 검토자 | jangster77, reviewer 할당 확인 |
| 관련 issue | [#4680](https://github.com/edwardkim/rhwp/issues/4680), 부분 수정 참조이며 Closes 아님 |
| 원 code candidate | `71f3cc5566c0aee366181c84de138d3d4e68809e` |
| 기준 devel | `6eb724481a1c41165c91fe9c31c959c4c3d0f28d` (#7071 병합) |
| 정렬·로컬 검증 head | `cf833c7a6413b4688ef9cd1fbf9777d16f00a174` |
| 검토 branch | `review/pr7067-20260913`, 주 작업공간 `/Users/tsjang/rhwp` |
| source branch | `planet6897/rhwp:fix/4680-hwp3-paragraph-style-id`, maintainer_can_modify=true |
| 실제 PR diff | source 1개·test 1개, +144 / -0; current-base merge 자체는 기능 변경에 포함하지 않음 |
| merge tree | `eabafb91ac5b9331f5941c493e54d22670895da4`, 자동 병합 충돌 없음 |
| 생성 시점 상태 | non-draft, MERGEABLE / CLEAN, 정렬 head의 checks 전부 통과 |
| 기본 경로 | collaborator_external_pr — 사용자가 해당 PR의 최신 head 정렬과 검토를 요청한 direct source 경로 |
| 보조 경로 | intake_and_review, local_validation, multi_pr_update_branch, review_only_fast_pass |

원 source SHA를 고정한 `update-branch`로 최신 devel을 병합했다. 정렬 head의 두 부모는 원 code
candidate와 위 devel SHA이며 contributor commit은 rebase/amend/force-push하지 않았다.
메인터너의 제품 source/test 보정은 없고, 이후 기록은 같은 branch의 문서-only trailing commit이다.

## 코드·계약 검토

`Hwp3ParaInfo::read`는 문단 레코드에서 `style_index: u8`를 읽고 있었다. 스타일 변환 루프는
HWP3 스타일 순서를 유지해 `doc_styles`에 append하며 별도 선두 스타일을 삽입하지 않는다.
따라서 `parse_paragraph_list`가 그 인덱스를 `Paragraph.style_id`에 그대로 옮기는 변경은
입력에 이미 있는 참조를 보존한다. 샘플 ID 분기·clamp·기준값 변경이 없다.

공통 문단 parser가 본문과 재귀 문단 경로에서 호출된다. HWP5 serializer의 `write_u8(para.style_id)`,
HWPX/HML의 스타일 참조 출력도 확인했다. paragraph/character shape의 직접 서식은 바꾸지 않는다.
스타일 참조를 비교하는 기존 typeset 복제 판정은 그대로이며 HWP3 전용 renderer 분기는 추가되지 않았다.
`Hwp3CharScan` destructuring의 두 `..`는 현재 field들을 모두 명명한 상태에서 동작 차이를 만들지 않는다.

기대값은 repository에 보존된 HWP3 원본과 한/글 HWP5 변환본 네 쌍의 본문 최상위 문단 참조다.
테스트는 문단 수와 위치별 style_id를 대조한다. table cell은 파서/한컴 변환본의 문단 수가 다른 별개
결함 때문에 pair 대조에서 제외되지만, table cell까지 순회하는 스타일 풀 범위 검사는 수행한다.
헤더·도형 등 모든 컨트롤을 순회하는 전수 검증으로 확대해 주장하지 않는다.

## 실제 검증

- macOS, Rust 1.93.1, 검증 head `cf833c7a6`. 기존 target 소유·활성 작업을 확인하고 전용
  `target/pr7067-review`를 새로 사용했다. shared target/debug·release·pr-review는 삭제하지 않았다.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 뒤 다음 focused 명령을 실행했다.

```bash
node scripts/run-rust-test.mjs issue_4680_hwp3_paragraph_style_reference --   --cargo-profile release-test --target-dir target/pr7067-review
```

- 결과: **3 PASS / 184 filtered skip**, exit 0, test 실행 3.103초(최초 빌드 3분 06초).
  본문 참조 일치·기본 스타일 외 참조·스타일 풀 범위의 세 검사가 통과했다.
- 음성 대조: repository source를 바꾸지 않고 원 test 파일의 임시 사본에서 HWP3 비교 벡터만
  `0`으로 치환했다. HWP5 기대값은 그대로 유지했다. 같은 검증 head의 rlib로 임시 harness를
  컴파일한 결과 **2 FAIL / 1 PASS**, exit 101로 예상대로 거부됐다. sample16 문단 index 5의
  `0 vs 6`과 사용 스타일 `{0}`를 검출했다. 이는 이전 결함의 출력에 해당하는 메모리 변이이며
  과거 source checkout의 전체 회귀 실행으로 기록하지 않는다.
- `cargo fmt --all -- --check`, suite manifest check, `git diff --check` 통과.
- 원 code candidate의 [Full CI 34700096414](https://github.com/edwardkim/rhwp/actions/runs/34700096414)에서
  lint와 Archive A~D, Build & Test가 실제 success였음을 Jobs API로 확인했다.
  source/test 보정 없이 current-base 자동 merge tree를 검증한 재검토이므로 local_validation 4.3.0에
  따라 같은 광범위 로컬 전체 회귀·Clippy를 반복하지 않았다. 로컬에서 실행하지 않은 검사를 PASS로 세지 않는다.
- contributor가 본문에서 언급한 Windows CRLF golden 실패를 재현·해소했다고 주장하지 않는다.
  수용 근거는 실제 GitHub Full CI와 위 Mac focused 검증이다.

## CI 정상 동작 확인

정렬 head의 실행은 아래와 같다. 빠른 success를 Full 재실행으로 오인하지 않았다.

| 검사 | 정렬 head 실행과 결과 |
| --- | --- |
| CI | [34706922273](https://github.com/edwardkim/rhwp/actions/runs/34706922273), success |
| CodeQL | [34706922269](https://github.com/edwardkim/rhwp/actions/runs/34706922269), success |
| Adapter | [34706922298](https://github.com/edwardkim/rhwp/actions/runs/34706922298), success |
| Proptest | [34706922264](https://github.com/edwardkim/rhwp/actions/runs/34706922264), success |
| CI Impact Policy | [34707029865](https://github.com/edwardkim/rhwp/actions/runs/34707029865), success |

CI와 CodeQL preflight는 candidate `71f3cc556`, `current-base-merge-tree-match`를 확인해 fast-pass했다.
이전 code candidate는 같은 PR/branch/repository의 실제 Full 실행으로 성공했으며, 정렬 commit은
current base와의 자동 병합 tree와 일치했다. CI lint/Archive worker와 CodeQL 분석 worker의 skip,
최종 aggregate success는 현행 재사용 계약에 따른 정상 동작이다. base의 source 변경까지 다시
Full로 실행했다고 주장하지 않는다. policy 상태는 exact head와 base `6eb724481`에 묶였다.

## 검증 입력의 commit 포함 확인

아래 실행 파일 8개는 모두 정렬 head의 commit blob 또는 LFS oid와 일치했다. Downloads 경로의
비공개 자료에 의존하지 않았다. 이 검토는 PDF 화면 비교를 수행하지 않았으며 새 PDF는 없다.

| 경로 | bytes | SHA-256 |
| --- | ---: | --- |
| `samples/hwp3-sample16.hwp` | 2949937 | `559fd94860cf836d5800436d56055c23a73e59eedef3da273a5ba365e6ed9e16` |
| `samples/hwp3-sample16-hwp5.hwp` | 3043328 | `6a3cdf2c148bf39f40ab06e847767e7f722543fce59e6ee4328ef44b835f3f45` |
| `samples/hwp3-sample11.hwp` | 391507 | `51b743b2823a2df9b6fac243f56aebecedbbd02e2a8baad58ffc2e5a4e695f20` |
| `samples/hwp3-sample11-hwp5.hwp` | 587264 | `412956ee85313584dcb901e162a19f68b70a35d57cbe76c95e5e3a1c6f591b9d` |
| `samples/hwp3-sample10.hwp` | 945360 | `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48` |
| `samples/hwp3-sample10-hwp5.hwp` | 1136128 | `a660a0d41898c8316479392c9687d81a15555431a581bdda988bf3598f6f0d51` |
| `samples/hwp3-sample19.hwp` | 25471 | `8da8f99201ca7ff2c7fe388f1ceafb61c86d556555c19affa59906d915cd6bd7` |
| `samples/hwp3-sample19-hwp5.hwp` | 29184 | `83e2ef560f7bbf10f44ed5ace53eefdb38576b04091ee96cc1bad5370f79e367` |

## 공통 조판 원칙 검토

| 항목 | 판정·근거 |
| --- | --- |
| 구현 근거·일반성 | 충족 — 레코드 인덱스와 스타일 풀 순서 보존, 기존 한컴 변환본 대조, 샘플 전용 분기 없음 |
| 측정·배치 일관성 | 비해당 — shape/LineSeg/좌표 계산 변경이 아닌 parser 참조 보존 |
| 줄 소속·점유 높이 | 비해당 — 줄 구성·높이·바깥여백 로직 변경 없음 |
| 사례·증거 독립성 | 충족 — committed HWP5 변환본 기대값, 네 쌍 대조와 zero-reference 음성 대조 |
| 기준값 변경 | 비해당 — golden/baseline/허용치 변경 없음 |
| 주장·검증 범위 | 충족 — 본문 스타일 참조에 한정; 한컴 개방·264쪽 문서·전체 HWP3 38건·PDF 일치는 미검증 |

이 PR은 페이지·줄바꿈·출력 외형 개선을 수용 근거로 주장하지 않는다. parser/serializer 참조 보존
범위이므로 이번 review에 visual sweep을 기계적으로 추가하지 않았다. #4680의 남은 필요조건은 계속 추적한다.

## Merge 후 contributor PR comment 계획

별도 병합 판단을 거쳐 이 PR을 실제 merge하면 merge SHA, 최종 head CI/fast-pass 근거,
로컬 3 PASS와 음성 대조, 위 committed 입력 8개 및 review permalink를 남긴다. 기여에 감사하고,
#4680 전체 종료 조건은 해결하지 않았으므로 OPEN 유지 사실을 명시한다. PDF 시각 일치나 한컴
개방 문제 해결을 주장하지 않는다. devel 동기화 뒤 local review branch·전용 target은 소유·활성 작업을
확인해 정리하며 contributor fork branch는 삭제하지 않는다.
