---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7157_review.md
last_verified: 2026-09-15
---

# PR #7157 검토

## 최종 판정

**승인**. HWP3 코드 25의 제목 차례 표식을 title_marks와 원시 UTF-16 오프셋에 보존한다. 독립 한컴 HWP5 대응본의 표식 수, 본문 레코드 존재, 저장·재열기 오프셋을 확인한 3개 테스트가 통과했다.

이 판정은 로컬 코드 검토이며 GitHub approve 제출이나 merge 완료가 아니다. 통합 batch의 #7165/#7168 코드·증거 보류도 메인터너 보정으로 해소했다.
통합 PR #7171의 최종 code head `5a04e1722` 원격 CI가 성공했다. 아래 최종 CI 기록을 참조한다.

## 대상과 출처

| 항목 | 검토값 |
| --- | --- |
| 원 PR | [#7157](https://github.com/edwardkim/rhwp/pull/7157) |
| 제목 | 수정: HWP3 제목 차례 표시를 버리지 않는다 (#4680) |
| 작성자 | planet6897, 기존 기여자 |
| 원 head | `6337bcefcca32adc0cf68a4436f504ffec9e289d` |
| PR base / 규모 | devel / 7 files, +293/-10 |
| 통합 base | `da7ec5c0906b38b7a1cfa15cf70ca5534b7d7ebb` |
| 검토 branch | `codex/pr7155-7167-review-20260915` |
| 적용 commit | `536f06f1f` |
| reviewer | jangster77 요청·API 재확인 완료 |
| 상태 | 작성 시점 OPEN/non-draft/MERGEABLE/CLEAN; 확인 시점 pending/failed 없음 |
| 검증 소스 / PDF 증거 commit | `b5d7956f6` / `925cd7434`(실행 코드 변화 없음) |

순서는 #7155 → #7157 → #7165 → #7166 → #7167 → #7168이며 각 고유 commit을 `-x`로 적용했다.
#7141/#7118 draft는 제외했다. [공통 실행 계획](pr_7155_7168_review_impl.md)에 전체 SHA와 순서를 기록했다.
base route는 `collaborator_external_pr.md`, modifier는 `multi_pr_update_branch.md`다.
`intake_and_review.md`, `maintainer_general.md`, `local_validation.md`, `visual_fixture_evidence.md`의 정본을 적용했다.

## 변경·검토 결과

선행 #7155의 공통 scan 상태를 사용하므로 원 PR의 중복 a00b736fa는 재적용하지 않았다. 실제 unique head는 6337bcef다. PR 본문에 남은 f05d1094f는 현재 head가 아니며 그 SHA를 검증값으로 사용하지 않았다. 코드 25 payload 4바이트를 소비하고 TitleMark의 char_idx는 visible 축, char_offsets/char_count는 8 UTF-16 유닛의 저장 축으로 유지한다. serializer가 title_marks에서 control_mask bit 8을 재구성하므로 parser의 mask만 보고 누락으로 판단하지 않았다. 한컴 대응본과 같은 제목 표식 범위를 확인했으며 표·그림 차례의 별도 의미 확장까지 완료했다고 주장하지 않는다.

관련 범위: #4680의 제목 차례 손실만 처리한다. 전체 이슈는 Ref 관계이며 닫지 않는다.

## 검증

`issue_4680_hwp3_title_mark_preservation`: 3 passed.

전용 target은 `target/pr7155-7167-review-20260915`다. 기존 공용 산출물은 삭제하지 않았다.
Native와 WASM은 #7168까지 포함한 소스에서 새로 빌드했다. `git diff da7ec5c0906b38b7a1cfa15cf70ca5534b7d7ebb...HEAD --check` 성공.
사용자 지시에 따라 광범위 전체 회귀를 중복 실행하지 않았으며 아래 source CI와 로컬 집중 검증을 구분한다.
통합 PR #7171의 code head `5a04e1722` Full CI가 성공했으며, source CI를 통합 완료로 대체하지 않는다.

- [Cancel stale PR runs](https://github.com/edwardkim/rhwp/actions/runs/34963085287/job/104361056665)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34963087364/job/104361142981)
- [CI](https://github.com/edwardkim/rhwp/actions/runs/34963087215/job/104364936286)
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/34963085283/job/104361057468)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34963087310/job/104361151731)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34963087292/job/104361144591)

## 공통 조판 원칙

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 사양·기존 serializer·독립 한컴 대응 입력의 구조 보존 테스트를 대조했다. |
| 측정·배치 일관성 | 비해당 | 파서의 제어/offset 보존이며 새 조판·분할 정책이나 baseline 변경은 없다. |
| 분할·이어받기 계약 | 비해당 | 파서의 제어/offset 보존이며 새 조판·분할 정책이나 baseline 변경은 없다. |
| 줄 소속과 점유 높이 | 비해당 | 파서의 제어/offset 보존이며 새 조판·분할 정책이나 baseline 변경은 없다. |
| 사례와 증거의 독립성 | 충족 | 사양·기존 serializer·독립 한컴 대응 입력의 구조 보존 테스트를 대조했다. |
| 기준값 변경 | 비해당 | 파서의 제어/offset 보존이며 새 조판·분할 정책이나 baseline 변경은 없다. |
| 주장과 검증 범위 | 충족 | 실행 검증과 사양 검토, 이슈의 미완료 범위를 구분했다. |

## 검증 입력과 증적

기존 입력은 이름을 바꿔 재추가하지 않았다. 새 PDF 2개는 `925cd7434`에 커밋했다.

| 저장소 파일 | SHA-256 | 마지막 입력 commit |
| --- | --- | --- |
| [samples/hwp3-sample10.hwp](../../../samples/hwp3-sample10.hwp) | `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48` | `e184718b10599266103a1137e58f0a5430fa22e3` |
| [samples/hwp3-sample10-hwp5.hwp](../../../samples/hwp3-sample10-hwp5.hwp) | `a660a0d41898c8316479392c9687d81a15555431a581bdda988bf3598f6f0d51` | `e184718b10599266103a1137e58f0a5430fa22e3` |

[검증 manifest](../assets/pr7155_7168_review_evidence.json)에 원 head의 CI 스냅샷, 바이너리·입력·PNG 해시와 sweep 수치를 보존한다. macOS Chrome webfont rasterizer, 96dpi를 사용했다.

## Merge 전 조건

통합 보류 사유는 메인터너 보정으로 해소됐고, 사용자로부터 PR 생성·CI 모니터링·merge·후속처리 승인을 받았다. 통합 최종 head의 CI 및 mergeability를 확인한 뒤 병합하며 원 PR은 직접 merge하지 않고 통합 후 supersede close한다.

## Merge 후 contributor PR comment 계획

[통합 PR #7171](https://github.com/edwardkim/rhwp/pull/7171)의 실제 merge SHA, 원 head,
제목 차례 표식·UTF-16 오프셋 및 저장 왕복 보존, 집중 검증과 통합 CI 결과, 이 review의 merge SHA 고정 링크를 남긴다.
감사와 함께 체리픽 통합된 사실을 설명하고 원 PR을 supersede close한다. #4680의 나머지
DocInfo 재색인·개방 문제는 종료하지 않는다. 이 PR은 구조 보존 검증이며 PDF visual 개선 수치를 주장하지 않는다.

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
