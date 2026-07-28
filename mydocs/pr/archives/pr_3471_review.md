# PR #3471 검토 — ir-diff 표 셀 재귀 비교

Issue: #3469
base route: maintainer_general
modifiers: intake_and_review, local_validation, rework_and_exceptions(supersede), post_merge

## 1. metadata (작성 시점 참고값, merge 전 재확인)

| 항목 | 값 |
|---|---|
| 제목 | fix(ir-diff): 표 셀 안의 변경을 감지하도록 셀 문단 재귀 추가 |
| author | kevin9327 (기존 contributor — merged 2, closed 32, open 6) |
| base | `devel` |
| head SHA | `baeb8d583` |
| 규모 | +406 -2, 5파일, commit 1개 |
| 연결 issue | #3469 (OPEN) |
| mergeable | MERGEABLE / **BEHIND** |
| CI | SUCCESS 20, SKIPPED 2 |

## 2. 중복 PR 판정 — #3470 supersede

**같은 이슈 #3469 를 같은 파일(`src/main.rs`)에서 고치는 PR 이 둘 있다.** planet6897 의
[#3470](https://github.com/edwardkim/rhwp/pull/3470)과 이 #3471 이다. 접근이 사실상 같고,
둘 다 #1807 이 글상자에 대해 닫은 구멍과 같은 계열임을 주석에 명시했다.

| | #3470 (planet6897) | #3471 (kevin9327) |
|---|---|---|
| 규모 | +161 -0, 2파일 | +406 -2, 5파일 |
| `src/main.rs` | +52 | +47 |
| 문단 비교 | 자체 로직 | **`diff_textbox_paragraph_lists` 재사용**(글상자와 동일 비교기) |
| 부가 증적 | 없음 | 보도자료 실사례 보고서 + PNG 2장 |
| 테스트 파일명 | `issue_3469_ir_diff_table_cells.rs` | `ir_diff_table_cells.rs` |

**작업지시자 판단으로 #3471 을 채택한다.** 기존 비교기를 재사용해 코드 중복이 적고, 실제
문서(보도자료 서식)에서 결함을 발견한 경위가 증적으로 남아 있다. #3470 은 supersede 로
close 한다.

## 3. 변경 범위

| 파일 | 변경 |
|---|---|
| `src/main.rs` | +47 — `diff_table` 에서 `diff_table_cells` 호출, 셀 수 → 문단 단위 재귀 |
| `tests/ir_diff_table_cells.rs` | 신규 — 표 셀 텍스트 변경 감지, 동일 문서 무차이, fill-fields 출력 경로 |
| `mydocs/report/edit_demo_hongbo/README.md` + PNG 2장 | 실사례 보고서 |

셀 문단 안의 중첩 표는 같은 경로를 다시 타므로 임의 깊이가 커버된다. 표 자체의 속성 비교는
`diff_table` 이 이미 하므로 이 함수는 내용만 본다.

## 4. 렌더 영향과 시각 검증 판정

**시각 검증 불필요.** [intake_and_review 2.6](../manual/pr_review/intake_and_review.md) 네
조건 중 어디에도 해당하지 않는다.

- `src/renderer`·`src/wasm_api.rs`·rhwp-studio Canvas 출력 경로 무변경 — CLI 의 diff 로직만 변경
- typeset·layout·paint·pagination 무관
- 기준 PDF·페이지 수·render-diff 주장 없음
- HWP/HWPX sample·golden·visual fixture 추가 없음

PNG 2장은 렌더 회귀 증적이 아니라 CLI 편집 실사례를 담은 **보고서 자산**이다. 보고서에
front matter 규격(`kind`/`status`/`canonical`/`last_verified`)이 갖춰져 있다.

## 5. 로컬 검증

검토 branch `review/kevin9327-3471-20260728` (기준 `origin/devel`).

| 검증 | 결과 |
|---|---|
| merge simulation | 충돌 0, `src/main.rs` auto-merge |
| focused `--test ir_diff_table_cells` | **3 passed / 0 failed** |
| **결함 검출 실증** | `src/main.rs` 를 수정 전으로 되돌리면 **2 FAILED** (exit 101) |
| `cargo test --profile release-test --tests` | **4233 passed / 0 failed** |
| `cargo fmt --check` / `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | **경고 0** |

### 테스트가 실제로 결함을 잡는지 실증했다

통과만 확인하면 테스트가 무엇을 지키는지 알 수 없다. `src/main.rs` 만 수정 전으로 되돌려
같은 테스트를 돌린 결과 `ir_diff_detects_text_change_inside_table_cells` 와
`fill_fields_default_output_lands_next_to_input` 이 FAILED 했다. 즉 이 테스트는 통과를
장식하는 것이 아니라 결함을 실제로 검출한다.

이 실증 과정에서 저장소 루트에 `rhwp-irdiff-cells-defaultout-*_filled.hwp` 가 남았다.
**PR 의 결함이 아니라 내 검증 방식의 부산물**이다 — 실패한 실행이 정리 코드(`remove_file`)에
도달하지 못했고, 당시 미수정 `fill-fields` 가 산출물을 현재 작업 디렉터리에 썼다. 정상
실행(수정된 코드)은 잔여물을 남기지 않는다. 확인 후 제거했다.

## 6. 코드 판단

- 글상자용 기존 비교기(`diff_textbox_paragraph_lists`)를 재사용한 선택이 타당하다. 표와
  글상자가 같은 "컨테이너 안 문단" 구조라 별도 로직을 두면 두 곳이 어긋날 수 있다.
- 주석에 #1807(글상자)·#1795(소거망 구멍) 선례를 명시해 이 수정이 같은 계열의 마지막 구멍을
  닫는 것임을 밝혔다.
- 셀 수가 다르면 그 사실만 보고하고 공통 구간만 비교하는 처리도 합리적이다. 길이 불일치를
  개별 셀 차이로 쏟아내면 노이즈가 된다.

발견한 문제·risk 는 없다.

## 7. 최종 권고

**merge 권고.** 결함이 실재하고(`--verify` 게이트가 표 손상을 통과시킴), 수정이 기존 구조를
재사용하며, 테스트가 결함을 실제로 검출함을 실증했다. 로컬 검증 전 항목 통과.

merge 전 조건:

- 최신 PR head 의 GitHub Actions 통과
- 작업지시자 승인
- BEHIND 상태 해소(update branch) 또는 admin merge 판단

후속: #3470 supersede close + 반영 위치 안내 코멘트, 이슈 #3469 close 확인.
