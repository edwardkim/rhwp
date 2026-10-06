# PR #6762 검토: lazy 기준 역산의 spacing_before 트림 복원

## 판정: 메인터너 보정 됨, 수용 가능

2026-09-05 갱신. 원 PR CI 조회 결과와 이번 로컬 실행 결과를 분리한다. 메인터너 보정은 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. 통합 PR은 작업지시자 승인에 따라 생성하는 단계이며 merge는 하지 않았다.

## PR 정보와 적용 이력

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#6762](https://github.com/edwardkim/rhwp/pull/6762) |
| 작성자 | `planet6897`, 기존 기여자 |
| 리뷰어 | `jangster77`, fetch·체리픽 전에 할당 |
| 원 head | `df03b740066dc96aa4ecb26665561d2d2f63d5a9` |
| base / draft | `devel` / `false` |
| 변경 규모 | 4개 파일, +230 / -2 |
| 원 head 조회 당시 병합 참고 상태 | `MERGEABLE` / `CLEAN` |
| 기준 devel | `2c144b180dd776aa450c499778510199ae6cdf89` |
| 로컬 검토 브랜치 | `review/ci-green-6759-6768-20260905` |
| 체리픽 커밋 | `09862f8f1`, 원본 출처를 `-x`로 보존 |
| 메인터너 보정 전 체리픽 HEAD | `d87b3037e5aeb6b662904b0182c361d5a2929108` |
| 메인터너 보정 commit | `902a208b515e83024502f004a2adaf84c33f18de` |

관련 [#6753](https://github.com/edwardkim/rhwp/issues/6753)은 양육수당 문서 5쪽의 본문 하한 초과와 6쪽 첫 줄 배분을 다룬다. 이번 base에는 선행 #6745가 이미 반영되어 있으므로 두 보정이 함께 있는 통합 상태가 검토 대상이다.

## 조회한 원 PR CI

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/33953587384/job/101274102267): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33954263843): `SUCCESS`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33953587143/job/101272780290): `SUCCESS`.
- CodeQL 분석 worker는 성공했지만 [플랫폼 CodeQL check](https://github.com/edwardkim/rhwp/runs/101274560976)의 원 결론은 `NEUTRAL`이다. 이를 `SUCCESS`로 바꾸어 기록하거나 알림을 자동 dismiss하지 않는다.
- 위 결과는 원 head 기준이다. 새 통합 head의 CI·로컬 검증 결과가 아니다.

## 보류 사유 해소

- 작은 음수의 0 보정보다 대체식 선택이 먼저 실행되던 문제를 수정했다. 공용 `resolve_lazy_base`에서 보정 후보를 먼저 정규화한 뒤 음수 여부로 대체식을 선택한다.
- `trimmed_spacing_before_px > 0.5`이고 후보가 `[-16, 0)`인 경우에만 0으로 정규화한다. 그 밖의 음수와 대체식 동작은 유지한다.
- 제품의 같은 helper를 integration test에서 호출하여 11개 경계 입력을 확인했다. 진단용 `lazy_base_corrected` 변수도 보존했다.
- 새 source-side `cfg(test)` module은 만들지 않았다. suite 정책의 해당 module 경로 허용만 한정했고 integration target 예산은 48/48로 유지했다.
- 실물 문서 테스트와 경계 테스트 두 건이 통과했고 최종 전체 Rust 회귀 9,049건도 통과했다.

## 직접 시각 대조와 한계

기존 `samples/issue6718/27469-child-allowance-retroactive-support.hwp`와 `pdf/issue6718-27469-2020.pdf`를 재사용했다. 한컴·rhwp 모두 12쪽이며 12쪽 각각의 문자 빈도 차이가 0이었다.

직접 연 5·6쪽 패널에서 5쪽 마지막 본문이 용지 안에 있고, 6쪽 첫 문장은 “비용의 지원을 신청할 수 있다.”로 확인했다. 6쪽 아래 인용 상자의 높이·여백과 글꼴 차이는 남으므로 전체 페이지의 시각 완전 일치로 판정하지 않는다.

추가 Studio canvas 캡처 스크립트는 첫 문서 페이지 준비 단계에서 중단되어 이 문서까지 도달하지 않았다. 완료된 native SVG/Chrome 대조와 실물 회귀 결과만 수용 근거로 사용한다.

## 원 PR·이슈 처리 범위

원 PR #6762와 #6753에는 메인터너가 수정한 분기 순서·경계 회귀와 5·6쪽 직접 대조 범위를 기록한다. 선행 #6745가 포함된 현재 기준에서의 결과이며 별도 선행 변경의 성과를 이 PR의 단독 성과로 바꾸지 않는다.

## 공통 검증과 승인 경계

전체 실행 명령, 첫 실패와 보정 후 결과, lint·Native Skia·WASM·Studio 결과는 [통합 검증 기록](pr_6759_review_impl.md)에 구분했다. 검증 대상은 체리픽 HEAD에 당시 미커밋 메인터너 보정을 더한 작업 트리였으며, 해당 보정은 이후 `902a208b515e83024502f004a2adaf84c33f18de`로 보존했다. 이를 순수 원 head 또는 최종 통합 PR CI 성공으로 대신 기록하지 않는다.

검토 판정은 GitHub approve·merge 권한 행사와 다르다. commit·push·통합 PR 생성은 작업지시자 승인 범위에서 진행한다. 최종 head CI와 시각 판정의 작업지시자 확인·merge 승인은 별도다. 현재 원격 comment·close·merge는 실행하지 않았다.

## Merge 후 댓글 작성 방식

승인된 통합 merge와 실제 devel CI 성공 뒤에만 [후속 처리 절차](../../manual/pr_review/post_merge.md)를 따른다. 원 PR 수용 출처·merge SHA·실제 PR/devel CI를 적고, 같은 merge SHA의 기존 댓글이 있으면 새 댓글 대신 수정한다. UTF-8 `--body-file`로 게시한 뒤 API로 body를 재조회한다.

아래 대표 PNG만 코멘트 이미지로 사용한다. `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr_6759_6768_20260905/<상대 PNG 경로>`를 Markdown 이미지로 넣어 댓글 안에서 직접 표시하고, [시각 대조 기록](pr_6759_6768_visual_sweep.md)과 해당 기준 PDF를 함께 연결한다. 존재하지 않는 merge SHA나 미완료 시나리오를 완료 증적으로 게시하지 않는다.

![#6762 검토 증적 1](../assets/pr_6759_6768_20260905/visual-6753/issue6753/review/review_005.png)

![#6762 검토 증적 2](../assets/pr_6759_6768_20260905/visual-6753/issue6753/review/review_006.png)
