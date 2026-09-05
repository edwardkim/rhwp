# PR #6768 검토: 후속 블록 표의 자리차지 표 배제 밴드 반영

## 판정: 메인터너 보정 됨, 수용 가능

2026-09-05 갱신. 원 PR CI 조회 결과와 이번 로컬 실행 결과를 분리한다. 메인터너 보정은 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. 통합 PR은 작업지시자 승인에 따라 생성하는 단계이며 merge는 하지 않았다.

## PR 정보와 적용 이력

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#6768](https://github.com/edwardkim/rhwp/pull/6768) |
| 작성자 | `planet6897`, 기존 기여자 |
| 리뷰어 | `jangster77`, fetch·체리픽 전에 할당 |
| 원 head | `31fdc395021e7974dca74feaa6f9ed3b3a395bf8` |
| base / draft | `devel` / `false` |
| 변경 규모 | 4개 파일, +151 / -0 |
| 원 head 조회 당시 병합 참고 상태 | `MERGEABLE` / `CLEAN` |
| 기준 devel | `2c144b180dd776aa450c499778510199ae6cdf89` |
| 로컬 검토 브랜치 | `review/ci-green-6759-6768-20260905` |
| 체리픽 커밋 | `d87b3037e`, 원본 출처를 `-x`로 보존 |
| 메인터너 보정 전 체리픽 HEAD | `d87b3037e5aeb6b662904b0182c361d5a2929108` |
| 메인터너 보정 commit | `902a208b515e83024502f004a2adaf84c33f18de` |

관련 [#6764](https://github.com/edwardkim/rhwp/issues/6764)의 블록 표 조각 23행이 용지 밖으로 나가는 축을 다룬다. 기여자는 후속 제목 문단 두 개의 초과가 남는다고 명시했으므로 이 PR을 issue 전체 해결로 취급하지 않는다.

## 조회한 원 PR CI

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/33957260691/job/101284087353): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33957946544): `SUCCESS`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33957260427/job/101282758750): `SUCCESS`.
- CodeQL 분석 worker는 성공했지만 [플랫폼 CodeQL check](https://github.com/edwardkim/rhwp/runs/101284541972)의 원 결론은 `NEUTRAL`이다. 이를 `SUCCESS`로 바꾸어 기록하거나 알림을 자동 dismiss하지 않는다.
- 위 결과는 원 head 기준이다. 새 통합 head의 CI·로컬 검증 결과가 아니다.

## 보류 사유 해소

- `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`를 출처·SHA-256·MANIFEST와 함께 등록하고 실물 회귀의 조용한 early return을 제거했다.
- 후속 블록 표가 선행 자리차지 표의 배제 밴드를 반영하는 좁은 수정이다. 같은 host의 표 및 HWPX에 무조건 확대하지 않는다.
- 표가 사라져도 경계 검사만 통과하지 않도록 별도 실물 회귀를 추가했다. rhwp 물리 183쪽에서 `rows=46, cols=3, pi=4, ci=0`인 표가 정확히 하나이고, source row 0~22 및 “1. 학습 및 동기부여”, “2. 팀(Team) 안에서의 상호 작용”이 존재하며 해당 표의 하단이 용지 안인지 확인한다.
- 새 sample 기준선 누락으로 최초 전체 회귀에서 text-overlap·off-canvas 두 건이 실패했다. 같은 문서로 실제 기준 devel 바이너리와 통합 후보를 대조한 뒤 새 경로에 한해서 text-overlap 23, off-canvas 5를 등록했다. 기존 문서 기준선은 상향하지 않았다.
- 최종 9,049개 전체 Rust 회귀와 표 존재·행 보존 회귀가 통과했다.

## 기준 devel 대비 측정

| 항목 | 기준 devel | 메인터너 보정 포함 후보 |
| --- | ---: | ---: |
| rhwp 전체 쪽수 | 200 | 201 |
| text-overlap | 23 | 23 |
| off-canvas | 6 | 5 |
| 표 하단 최대 용지 초과 | 885.5567 px | 13.3567 px |

이 수치는 샘플 SHA-256 `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`에 대한 측정이다. baseline 파일의 필드는 문서별 탐지 건수이며 전체 페이지 수가 아니다.

## 시각 대조와 수용 범위

한컴 2018 저장본이므로 비동기 MCP `engine 2020`으로 만든 204쪽 PDF를 기준으로 삼았다. 학습 표 내용이 대응하는 rhwp 물리 183쪽과 PDF 물리 186쪽을 대조했다. 원 PDF를 수정하지 않고 대조용 페이지 매핑만 적용했다.

직접 연 패널에서 대상 표가 rhwp 용지 안에 있고 학습·팀 항목이 보인다. PDF의 선행 표 꼬리와 제목 위치, 행의 쪽 배분은 다르다. rhwp 물리 182쪽 제목 “과목 2: 인적 요소”의 61.4667 px 초과는 기준 devel에서도 동일하며 남아 있다. 그 밖의 잔여 표 초과는 물리 136·151·168쪽에서 각각 약 2.0767·9.1167·13.3567 px다.

따라서 **PR의 큰 표 조각 초과 해소 축만 수용 가능**하다. #6764 전체 해결 또는 한컴 204쪽과의 전체 페이지 동일성을 주장하지 않는다. 추가 Studio canvas 자동화는 이 문서에 도달하지 못했으므로 native 대조 결과만 기록한다.

## 원 PR·이슈 처리 범위

원 PR #6768에는 표 축의 부분 수용을 기록한다. #6764에는 동일한 증적과 잔여 제목·쪽 배분 문제를 설명하되 **issue를 닫지 않는다**. 통합 PR에도 #6764 전체 종료를 유발하는 closing keyword를 넣지 않는다.

## 공통 검증과 승인 경계

전체 실행 명령, 첫 실패와 보정 후 결과, lint·Native Skia·WASM·Studio 결과는 [통합 검증 기록](pr_6759_review_impl.md)에 구분했다. 검증 대상은 체리픽 HEAD에 당시 미커밋 메인터너 보정을 더한 작업 트리였으며, 해당 보정은 이후 `902a208b515e83024502f004a2adaf84c33f18de`로 보존했다. 이를 순수 원 head 또는 최종 통합 PR CI 성공으로 대신 기록하지 않는다.

검토 판정은 GitHub approve·merge 권한 행사와 다르다. commit·push·통합 PR 생성은 작업지시자 승인 범위에서 진행한다. 최종 head CI와 시각 판정의 작업지시자 확인·merge 승인은 별도다. 현재 원격 comment·close·merge는 실행하지 않았다.

## Merge 후 댓글 작성 방식

승인된 통합 merge와 실제 devel CI 성공 뒤에만 [후속 처리 절차](../../manual/pr_review/post_merge.md)를 따른다. 원 PR 수용 출처·merge SHA·실제 PR/devel CI를 적고, 같은 merge SHA의 기존 댓글이 있으면 새 댓글 대신 수정한다. UTF-8 `--body-file`로 게시한 뒤 API로 body를 재조회한다.

아래 대표 PNG만 코멘트 이미지로 사용한다. `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr_6759_6768_20260905/<상대 PNG 경로>`를 Markdown 이미지로 넣어 댓글 안에서 직접 표시하고, [시각 대조 기록](pr_6759_6768_visual_sweep.md)과 해당 기준 PDF를 함께 연결한다. 존재하지 않는 merge SHA나 미완료 시나리오를 완료 증적으로 게시하지 않는다.

![#6768 검토 증적 1](../assets/pr_6759_6768_20260905/mapped-6764-rhwp183-pdf186/review/review_183.png)
