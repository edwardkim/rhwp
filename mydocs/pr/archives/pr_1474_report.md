# PR #1474 사전 처리 판단 보고서 - ir-diff tab_extended 예약 필드 노이즈 제거

- PR: https://github.com/edwardkim/rhwp/pull/1474
- 제목: `fix(ir-diff): tab_extended 예약 필드[3,4,5] 거짓 차이 제외 (#1473)`
- 작성일: 2026-06-22
- 컨트리뷰터: `oksure` (Hyunwoo Park)
- 관련 이슈: #1473
- 처리 경로: collaborator-mediated 외부 PR 경로
- 보고서 성격: merge 전 사전 처리 판단 보고서

이 문서는 PR head에 함께 포함하기 위한 사전 판단 보고서다. 아직 merge가 완료되지 않았으므로
merge SHA, 실제 merge 시각, 이슈 close 완료 여부는 기록하지 않는다. 최종 사실 기록은 GitHub PR/Issue
metadata를 원천으로 삼는다.

## 1. 처리 판단

**merge 수용 권고.**

PR #1474는 `ir-diff`의 HWPX↔HWP5 탭 확장 비교에서 포맷 비대칭 슬롯을 제외해 false positive를 줄이는
도구 변경이다. 변경 범위가 `src/main.rs`의 진단 경로에 한정되어 있고, 파서·직렬화·렌더러의 보존 동작을
직접 변경하지 않는다.

## 2. 수용 근거

- #1473에서 보고된 `tab_extended[1]`, `[3]`, `[4]`, `[5]` 포맷 비대칭 노이즈를 `ir-diff` 출력에서 제외한다.
- 의미 비교 대상은 `[0]` width, `[2]` type/leader pack, `[6]` marker로 유지되어 실제 탭 폭·종류·leader 차이는 계속 검출된다.
- 탭 개수 비교는 기존대로 유지된다.
- Copilot 리뷰 후 작성자가 `ext[1]`의 HWPX/HWP5 비대칭을 재검토해 최종 커밋에 반영했다.
- 대표 샘플에서 PR 본문 주장과 같은 결과를 확인했다.

## 3. 검증 요약

로컬 검증:

| 항목 | 결과 |
|---|---|
| `git diff --check upstream/devel...FETCH_HEAD` | 통과 |
| `cargo test --bin rhwp tab_ext` | 2 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --bin rhwp -- -D warnings` | 통과 |
| 대표 `3-11월...` HWPX/HWP `ir-diff --summary` | `indent` 14건만 남음 |
| `pic2` HWPX/HWP `ir-diff --summary` | 차이 0건 |
| `table-vpos-01` HWPX/HWP `ir-diff --summary` | 차이 0건 |

GitHub Actions는 작성 시점에 Build & Test, CodeQL, Render Diff가 통과 상태였으나, 본 문서 커밋을 PR head에
추가하면 재실행될 수 있다. merge 전 최신 head 기준으로 다시 확인한다.

## 4. 비차단 리스크

`tab_extended[1]`은 렌더 보조 경로에서 leader/fill 후보로 읽히는 코드가 남아 있다. 따라서 이 PR을
"코드 전체에서 [1]이 무의미하다"는 일반 모델 정정으로 해석하면 안 된다.

이번 판단 범위는 `ir-diff`의 HWPX↔HWP5 parity 비교에서 HWPX가 표현하지 못하거나 HWP5 전용인 슬롯을
false positive에서 제외하는 데 한정한다. inline tab leader 렌더 모델 정리는 필요 시 별도 이슈에서 다룬다.

## 5. merge 전 최종 조건

1. 본 review/report/workflow 문서 커밋이 PR #1474 head에 포함된다.
2. 문서 커밋 push 후 최신 GitHub Actions가 통과한다.
3. GitHub review 또는 PR comment로 검토 결과를 남긴다.
4. 작업지시자가 merge를 승인한다.

## 6. merge 후 확인 항목

1. #1473 자동 close 여부를 확인한다.
2. `devel`이 default branch가 아니어서 자동 close가 실패하면, 작업지시자 승인 후 수동 close한다.
3. contributor에게 검증 결과와 merge 처리 사실을 코멘트로 남긴다.
