# PR #6759 검토: 선택 복원 소스 계약의 주석 오탐 방지

## 판정: 승인

2026-09-05 갱신. 원 PR CI 조회 결과와 이번 로컬 실행 결과를 분리한다. 메인터너 보정은 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. 통합 PR은 작업지시자 승인에 따라 생성하는 단계이며 merge는 하지 않았다.

## PR 정보와 적용 이력

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#6759](https://github.com/edwardkim/rhwp/pull/6759) |
| 작성자 | `lpaiu-cs`, 기존 기여자 |
| 리뷰어 | `jangster77`, fetch·체리픽 전에 할당 |
| 원 head | `4071ae534bd380a18c41125284f96c1b5a6a7e15` |
| base / draft | `devel` / `false` |
| 변경 규모 | 1개 파일, +15 / -6 |
| 원 head 조회 당시 병합 참고 상태 | `MERGEABLE` / `CLEAN` |
| 기준 devel | `2c144b180dd776aa450c499778510199ae6cdf89` |
| 로컬 검토 브랜치 | `review/ci-green-6759-6768-20260905` |
| 체리픽 커밋 | `86f79ec31`, 원본 출처를 `-x`로 보존 |
| 메인터너 보정 전 체리픽 HEAD | `d87b3037e5aeb6b662904b0182c361d5a2929108` |
| 메인터너 보정 commit | `902a208b515e83024502f004a2adaf84c33f18de` |

관련 이슈 [#3416](https://github.com/edwardkim/rhwp/issues/3416)는 조회 시 CLOSED였다. 이번 변경은 선택 복원 제품 동작 자체가 아니라 기존 테스트의 주석 오탐·거짓 통과를 막는 범위다.

## 조회한 원 PR CI

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/33957707151/job/101284166160): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33957856966): `SUCCESS`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33957707080/job/101283956505): `SKIPPED` (제품·렌더 변경 없는 테스트 전용 범위)
- CodeQL 분석 worker는 성공했지만 [플랫폼 CodeQL check](https://github.com/edwardkim/rhwp/runs/101284309669)의 원 결론은 `NEUTRAL`이다. 이를 `SUCCESS`로 바꾸어 기록하거나 알림을 자동 dismiss하지 않는다.
- 위 결과는 원 head 기준이다. 새 통합 head의 CI·로컬 검증 결과가 아니다.

## 검토 결과

- 제품 소스는 바꾸지 않고 네 소스 계약 검사의 입력을 `codeOnly`로 제한한다. 주석에 남은 문자열로 계약이 거짓 통과하거나 오탐되는 문제를 줄인다.
- 기존 실행 동작 테스트 두 개는 유지했다. 통합 Studio 테스트 1,403건 통과, 1건 skip, 실패 0건으로 확인했다.
- 제품 렌더 변경이 없으므로 이 PR만을 위한 시각 이미지를 추가로 만들지 않는다. 다른 PR의 렌더 이미지를 이 변경의 동작 증명으로 게시하지 않는다.
- 새로운 메인터너 제품 보정은 필요하지 않았다. 이 판정은 로컬 검토 결과이며 원 PR의 GitHub approve 또는 merge를 실행했다는 뜻이 아니다.

## 원 PR·이슈 처리 범위

원 PR #6759에는 통합 수용 출처와 실제 검증 결과를 기록한다. 이미 CLOSED인 #3416을 이번 변경으로 새로 해결했다고 주장하거나 중복 close하지 않는다.

## 공통 검증과 승인 경계

전체 실행 명령, 첫 실패와 보정 후 결과, lint·Native Skia·WASM·Studio 결과는 [통합 검증 기록](pr_6759_review_impl.md)에 구분했다. 검증 대상은 체리픽 HEAD에 당시 미커밋 메인터너 보정을 더한 작업 트리였으며, 해당 보정은 이후 `902a208b515e83024502f004a2adaf84c33f18de`로 보존했다. 이를 순수 원 head 또는 최종 통합 PR CI 성공으로 대신 기록하지 않는다.

검토 판정은 GitHub approve·merge 권한 행사와 다르다. commit·push·통합 PR 생성은 작업지시자 승인 범위에서 진행한다. 최종 head CI와 시각 판정의 작업지시자 확인·merge 승인은 별도다. 현재 원격 comment·close·merge는 실행하지 않았다.

## Merge 후 댓글 작성 방식

승인된 통합 merge와 실제 devel CI 성공 뒤에만 [후속 처리 절차](../../manual/pr_review/post_merge.md)를 따른다. 원 PR 수용 출처·merge SHA·실제 PR/devel CI를 적고, 같은 merge SHA의 기존 댓글이 있으면 새 댓글 대신 수정한다. UTF-8 `--body-file`로 게시한 뒤 API로 body를 재조회한다.

아래 대표 PNG만 코멘트 이미지로 사용한다. `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr_6759_6768_20260905/<상대 PNG 경로>`를 Markdown 이미지로 넣어 댓글 안에서 직접 표시하고, [시각 대조 기록](pr_6759_6768_visual_sweep.md)과 해당 기준 PDF를 함께 연결한다. 존재하지 않는 merge SHA나 미완료 시나리오를 완료 증적으로 게시하지 않는다.

이 PR에는 별도 시각 이미지가 필요하지 않으며 실행 결과를 글로 기록한다.
