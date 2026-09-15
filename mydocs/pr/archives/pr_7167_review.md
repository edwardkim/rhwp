---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7167_review.md
last_verified: 2026-09-15
---

# PR #7167 검토

## 최종 판정

**승인**. 표시 문구 대신 안정된 tab ID로 동작하도록 보정한 범위를 수용한다. #7166과의 두 충돌 및 번역 카탈로그를 인식하지 못하는 테스트를 해결했고, 실제 브라우저에서 번역·표시 문구 변경 후 패널 전환을 확인했다.

이 판정은 로컬 코드 검토이며 GitHub approve 제출이나 merge 완료가 아니다. 통합 batch의 #7165/#7168 코드·증거 보류도 메인터너 보정으로 해소했다.
통합 PR #7171의 최종 code head `5a04e1722` 원격 CI가 성공했다. 아래 최종 CI 기록을 참조한다.

## 대상과 출처

| 항목 | 검토값 |
| --- | --- |
| 원 PR | [#7167](https://github.com/edwardkim/rhwp/pull/7167) |
| 제목 | refactor(ui): 대화상자 탭을 이름 대신 ID 로 구분 (#5852) |
| 작성자 | rubidus-api, 기존 기여자 |
| 원 head | `361669bd8033df7e8aca6d06687796567a87c4af` |
| PR base / 규모 | devel / 10 files, +161/-58 |
| 통합 base | `da7ec5c0906b38b7a1cfa15cf70ca5534b7d7ebb` |
| 검토 branch | `codex/pr7155-7167-review-20260915` |
| 적용 commit | `416da982d + 메인터너 a86e96bb8` |
| reviewer | jangster77 요청·API 재확인 완료 |
| 상태 | 작성 시점 OPEN/non-draft/MERGEABLE/CLEAN; 확인 시점 pending/failed 없음 |
| 검증 소스 / PDF 증거 commit | `b5d7956f6` / `925cd7434`(실행 코드 변화 없음) |

순서는 #7155 → #7157 → #7165 → #7166 → #7167 → #7168이며 각 고유 commit을 `-x`로 적용했다.
#7141/#7118 draft는 제외했다. [공통 실행 계획](pr_7155_7168_review_impl.md)에 전체 SHA와 순서를 기록했다.
base route는 `collaborator_external_pr.md`, modifier는 `multi_pr_update_branch.md`다.
`intake_and_review.md`, `maintainer_general.md`, `local_validation.md`, `visual_fixture_evidence.md`의 정본을 적용했다.

## 변경·검토 결과

7종 대화상자와 2개 E2E의 탭 선택을 data-tab으로 바꾼다. 코드 검토에서 table/cell·picture·equation·char·para·page-border·cell-border의 ID 매핑과 각 탭의 적용 분기를 대조했다. `equation-props-dialog.ts`는 t import와 basic/margin/equation ID 선언을 함께 유지했다. `page-border-dialog.ts`는 border/background ID와 #7166의 번역 label을 함께 유지했다.

통합 직후 `탭 화면 글자는 그대로다` 검사가 번역 호출을 문자열 리터럴로 오인했다. `a86e96bb8`에서 기존 `assertShowsText`를 써 ko 카탈로그와 호출을 검사하도록 바꿨다. 표시 글자를 바꿔도 클릭한 ID의 패널이 나타나는지 실제 Chrome에서 추가 확인했다. 4종×2언어의 런타임과 기존 OLE 객체 선택 E2E를 실행했으며, 미실행 화면을 전체 검증했다고 쓰지 않는다.

관련 범위: #5852 번역 전제 정리이며 전체 이슈 종료 범위가 아니다.

## 검증

`dialog-tab-ids.test.ts`: 4 passed. Studio 전체 1740 pass / 2 skip, TypeScript 2종 성공. 실제 대화상자 4종×2언어 성공. `issue-2069-ole-object-selection.test.mjs` 성공. `undo-contracts.test.mjs`: 5개 동작 묶음 성공(모두 바꾸기·문단·글자·표 속성 undo 및 Through 보존).

전용 target은 `target/pr7155-7167-review-20260915`다. 기존 공용 산출물은 삭제하지 않았다.
Native와 WASM은 #7168까지 포함한 소스에서 새로 빌드했다. `git diff da7ec5c0906b38b7a1cfa15cf70ca5534b7d7ebb...HEAD --check` 성공.
사용자 지시에 따라 광범위 전체 회귀를 중복 실행하지 않았으며 아래 source CI와 로컬 집중 검증을 구분한다.
통합 PR #7171의 code head `5a04e1722` Full CI가 성공했으며, source CI를 통합 완료로 대체하지 않는다.

- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34967192355/job/104374449897)
- [CI](https://github.com/edwardkim/rhwp/actions/runs/34967192285/job/104377074720)
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/34967189615/job/104374342042)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34967192318/job/104374440186)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34967192308/job/104374429270)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34967192044/job/104374417475)

## 공통 조판 원칙

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 비해당 | UI 표시·탭 식별 변경이며 문서 조판·저장 줄·예산을 바꾸지 않는다. |
| 측정·배치 일관성 | 비해당 | UI 표시·탭 식별 변경이며 문서 조판·저장 줄·예산을 바꾸지 않는다. |
| 분할·이어받기 계약 | 비해당 | UI 표시·탭 식별 변경이며 문서 조판·저장 줄·예산을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | UI 표시·탭 식별 변경이며 문서 조판·저장 줄·예산을 바꾸지 않는다. |
| 사례와 증거의 독립성 | 비해당 | UI 표시·탭 식별 변경이며 문서 조판·저장 줄·예산을 바꾸지 않는다. |
| 기준값 변경 | 비해당 | UI 표시·탭 식별 변경이며 문서 조판·저장 줄·예산을 바꾸지 않는다. |
| 주장과 검증 범위 | 충족 | source/통합 SHA, unit·실제 UI 범위와 남은 번역 범위를 구분했다. |

## 검증 입력과 증적

빈 문서를 생성하는 실제 Studio 실행을 사용했다. 새 HWP/HWPX/PDF 입력은 없다. OLE E2E가 참조한 기존 샘플은 실행 계획의 입력 목록에 기록하며 중복 추가하지 않는다.

[검증 manifest](../assets/pr7155_7168_review_evidence.json)에 원 head의 CI 스냅샷, 바이너리·입력·PNG 해시와 sweep 수치를 보존한다. macOS Chrome webfont rasterizer, 96dpi를 사용했다.

![pr7167_page_border_en.png](../assets/pr7167_page_border_en.png)

## Merge 전 조건

통합 보류 사유는 메인터너 보정으로 해소됐고, 사용자로부터 PR 생성·CI 모니터링·merge·후속처리 승인을 받았다. 통합 최종 head의 CI 및 mergeability를 확인한 뒤 병합하며 원 PR은 직접 merge하지 않고 통합 후 supersede close한다.

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md)을 연결한다. 위 실제 페이지·수치·직접 관찰과 미검증 범위를 요약하고, 대표 PNG는 다음처럼 merge SHA로 고정한다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7167_page_border_en.png`

asset이 devel에 포함되고 merge SHA가 확정된 뒤, 허용된 게시 단계에서 `--body-file`로 작성하고 API로 본문을 재조회한다. 현재는 로컬 계획이며 게시하지 않았다. 보류 상태의 불일치 PNG를 개선 완료 증거로 게시하지 않는다.

OLE E2E 입력은 아래 두 기존 파일이며 직접 이름을 바꾸거나 추가하지 않았다.

| 파일 | SHA-256 | 입력 commit |
| --- | --- | --- |
| [samples/SO-SUEOP.hwpx](../../../samples/SO-SUEOP.hwpx) | `ebbd6f5c86d6eda195bb9c4ba1fc35b329432dcc5ef1dc0fffbfb4542fce74b8` | `7393203034a92fdbb241812458b539753eb01985` |
| [samples/한셀OLE.hwp](../../../samples/한셀OLE.hwp) | `fdc595a2f5f99f97653b91e57b4c74090391495ce5f84ef2e60308ea4b8e9da3` | `1dc5a5706bfaf31115f2455829e9b135607d050a` |

## 승인된 통합 경로

[PR #7171](https://github.com/edwardkim/rhwp/pull/7171), code candidate `5a04e172247d1b168d350faf514beeef46cf0964`.
base route: `collaborator_external_pr.md`; modifiers: `multi_pr_update_branch.md`,
`review_only_fast_pass.md`, `post_merge.md`. 개별 review·오늘할일·증적은 code CI 성공 뒤
같은 PR의 문서-only trailing commit으로 반영한다. 최종 head CI·mergeability를 확인한 뒤
merge하고 duration refresh·원 PR comment/close·devel 동기화·전용 산출물 정리를 수행한다.

## CI 정책 보정 후 최신 후보

최초 `95d631f0a`의 CI는 source-side 테스트 총량의 PR base 비교에서 실패했다.
Rust archive·Native Skia·Frontend package 검증은 성공했다. `5a04e172247d1b168d350faf514beeef46cf0964`에서
개체 동반 공백 반례를 기존 cursor 계약 안에서 동일하게 실행하도록 묶고 정책 총량 4205를
유지했다. 생산 코드·시각 출력은 바뀌지 않았다. base 비교·cursor 테스트·필수 lint 묶음을
재검증했고 최신 후보 Full CI 성공을 확인해 trailing 문서를 반영한다.

## 최종 code CI 확인 (2026-09-15)

최종 code head `5a04e172247d1b168d350faf514beeef46cf0964`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34978539946),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34978540233),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34978539840),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/34978539982),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/34978540234)가 성공했다.
확인 시 check 31개 성공·4개 조건부 생략, policy status 성공이며 실패·진행 중 항목은 없다.
Rust archive 4개·전체 shard, Native Skia, 필수 lint, frontend package 검증을 포함한다.
초기 `95d631f0a`의 source-side test 정책 실패는 `5a04e1722`에서 동일 assertion을 기존
테스트 계약으로 묶어 해결했다. 허용치·baseline·생산 코드는 변경하지 않았다.
이 문서·오늘할일·대표 증적을 single-parent trailing commit으로 반영하고, 문서 추가 후
최종 head의 fast-pass와 required checks·MERGEABLE/CLEAN을 별도로 확인한 뒤 병합한다.
최종 head 및 실제 merge SHA·후속 처리 결과는 PR #7171과 원 PR의 GitHub comment에 기록한다.
