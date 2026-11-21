# PR #9238 검토 — 글자 사이 조판 부호의 표시 위치

## 통합 후보의 검토 결과

**메인테이너 보정 후 수용 가능.** 편집 화면에서 두 글자 사이의 조판 부호를 켜고 끈 두 상태의 Print SVG가 바이트 단위로 일치한다. 공통 회귀와 보호 문서의 판정은 [17건 통합 검토](pr_9292_review.md)에 기록했다.

## 접수 당시 판정

**머지 보류.** 새 번호가 화면에서는 올바른 위치에 한 번 표시된다. 편집 화면에서 조판 부호를 켠 상태로 Print 출력을 요청하면 그 부호가 인쇄되고 뒤 글자의 위치까지 바뀐다.

## 접수 및 적용

- 원 기여는 `semanticist21/fix/control-code-marker-in-run`의 `debc6ac352b76de0a9071b5177a82ee92be2f8c0`이다. 리뷰어 `edwardkim` 요청은 2026-11-20 10:54:43 UTC에 확인했다.
- 기준 `upstream/devel`은 `0a437a7eef`이다. 11번째 누적 적용 결과는 `1ff46b9767`이며 원 head 단독과 앞선 10건과의 누적 병합에서 모두 충돌이 없었다.
- 최소 글꼴 환경의 Native release-test에서 `control_code_marker_position` 회귀 2개가 통과했다. [PR과 CI](https://github.com/edwardkim/rhwp/pull/9238/checks)

## 화면과 인쇄 대조

빈 문서에 `abcd`를 입력하고 글자 위치 2에서 새 번호를 삽입했다. 편집 중인 문서를 그대로 둔 채 조판 부호 표시를 껐다 켜면서 Screen과 Print 프로필의 SVG에서 각 글자의 위치를 읽었다.

| 상태 | Print에서 관측한 순서 | 글자 c의 x |
| --- | --- | ---: |
| 조판 부호 끔 | a, b, c, d | 128.91px |
| 켬, 이번 PR 적용 전의 10건 | a, b, c, d, 새 번호 부호 | 128.91px |
| 켬, 이번 PR 포함 | a, b, 새 번호 부호, c, d | 158.91px |

앞선 코드에도 인쇄에 부호가 남는 문제가 있었다. 이번 변경에서는 그 부호가 본문 가운데로 옮겨 c와 d의 인쇄 위치도 각 30px 변했다. Print 결과가 화면의 조판 부호 표시 여부와 무관하게 본문 순서와 좌표를 보존하는지 검사해야 한다. 화면에서 부호가 올바른 줄에 한 번 그려지는 두 회귀도 함께 유지한다.

## 2026-11-21 메인테이너 보완 착수

원 [PR #9238](https://github.com/edwardkim/rhwp/pull/9238)의 head는 `debc6ac352b76de0a9071b5177a82ee92be2f8c0`이다. GitHub에서 담당 리뷰어 `edwardkim`을 확인했다. 기준 devel은 `2480af27df`이다.

화면 조판 부호를 켜면 새 번호 뒤의 c·d가 명시적 Print 출력에서도 각각 30px씩 밀린다. Screen 표시를 Print의 문자 배치에서 분리하고 두 프로필의 글자 좌표를 독립 확인한다.

## 메인테이너 보완 검증 중간 결과

동일 문서에서 조판 부호 표시를 켜고 끈 Print SVG가 바이트 단위로 동일했다. Screen은 별도로 생성해서 비교했다. [쪽별 비교 원장](../assets/semanticist21-20261121/maintainer-comparison.tsv)

## 병합 후 기여자 PR 댓글 계획

두 글자 사이의 조판 부호가 화면의 제자리에 한 번 표시됩니다. 화면의 표시 옵션을 바꾸어도 인쇄에서 뒤 글자가 밀리지 않고, 켜고 끈 상태의 Print SVG가 바이트 단위로 같았습니다. 통합 PR과 확정된 병합 커밋, 이 PR의 검토 기록을 함께 연결한다.

## 통합 결과

2026년 11월 22일 02시 15분(KST)에 [통합 PR #9292](https://github.com/edwardkim/rhwp/pull/9292)를 `devel`에 병합했다. 원 기여 커밋 `debc6ac352b76de0a9071b5177a82ee92be2f8c0`이 병합 이력에 포함된 것을 확인했다. [원 PR #9238](https://github.com/edwardkim/rhwp/pull/9238) · [병합 커밋 `0adeca439f`](https://github.com/edwardkim/rhwp/commit/0adeca439f1481e7245a344b7713d7d563df2ace) · [전체 CI와 검토](pr_9292_review.md)
