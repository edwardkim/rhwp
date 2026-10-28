# PR #6760 검토: RowBreak 컷 벡터 범위 초과 시 행 단위 인덱스 복원

## 판정: 메인터너 보정 됨, 수용 가능

2026-09-05 갱신. 원 PR CI 조회 결과와 이번 로컬 실행 결과를 분리한다. 메인터너 보정은 `902a208b515e83024502f004a2adaf84c33f18de`로 커밋했다. 통합 PR은 작업지시자 승인에 따라 생성하는 단계이며 merge는 하지 않았다.

## PR 정보와 적용 이력

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#6760](https://github.com/edwardkim/rhwp/pull/6760) |
| 작성자 | `planet6897`, 기존 기여자 |
| 리뷰어 | `jangster77`, fetch·체리픽 전에 할당 |
| 원 head | `ac6c297c5aacf47e3358f401a5123f02ea8e7157` |
| base / draft | `devel` / `false` |
| 변경 규모 | 8개 파일, +164 / -0 |
| 원 head 조회 당시 병합 참고 상태 | `MERGEABLE` / `CLEAN` |
| 기준 devel | `2c144b180dd776aa450c499778510199ae6cdf89` |
| 로컬 검토 브랜치 | `review/ci-green-6759-6768-20260905` |
| 체리픽 커밋 | `b07744075`, 원본 출처를 `-x`로 보존 |
| 메인터너 보정 전 체리픽 HEAD | `d87b3037e5aeb6b662904b0182c361d5a2929108` |
| 메인터너 보정 commit | `902a208b515e83024502f004a2adaf84c33f18de` |

원 PR은 [#6756](https://github.com/edwardkim/rhwp/issues/6756)을 `Fixes`로 참조한다. 지정항로 문서의 2쪽 끝과 3쪽 머리에 행이 중복되고 글자가 용지 밖으로 나가는 문제다.

## 조회한 원 PR CI

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/33951549625/job/101268565159): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33952232899): `SUCCESS`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33951549521/job/101267227944): `SUCCESS`.
- CodeQL 분석 worker는 성공했지만 [플랫폼 CodeQL check](https://github.com/edwardkim/rhwp/runs/101269033639)의 원 결론은 `NEUTRAL`이다. 이를 `SUCCESS`로 바꾸어 기록하거나 알림을 자동 dismiss하지 않는다.
- 위 결과는 원 head 기준이다. 새 통합 head의 CI·로컬 검증 결과가 아니다.

## 보류 사유 해소

- `samples/issue6756/17253153-traffic-safety-designated-routes.hwp`를 출처·SHA-256·MANIFEST와 함께 정식 등록했다.
- 회귀 테스트가 환경 변수나 기여자 Windows 경로를 찾다가 조용히 반환하던 동작을 제거했다. 필수 sample 또는 render tree 생성에 실패하면 테스트도 실패한다.
- 컷 벡터의 블록 인덱스가 범위를 벗어나는 경우에만 해당 단일 행의 인덱스로 복구하는 제품 수정 범위를 유지했다.
- 두 focused 회귀가 최종 전체 실행에서 통과했고, 전체 Rust 회귀 9,049건도 통과했다.

## 직접 시각 대조와 한계

한컴 저장 정보는 2020이므로 MCP `engine 2020` 기준 PDF를 비동기로 생성했다. 한컴·rhwp 모두 5쪽이다. 실제로 연 비교 패널에서 rhwp 2쪽 끝의 항목 1이 3쪽 머리에 반복되지 않고, 3쪽은 항목 2에 이어 3·4로 진행하며 선택 페이지의 용지 하단 초과가 보이지 않았다.

한컴 PDF는 2쪽에 항목 1·2, 3쪽에 3·4를 배분하므로 행의 쪽 배분까지 동일하지는 않다. 전체 페이지를 합친 문자 빈도 차이는 0이지만 이것을 문자 순서 또는 픽셀 완전 일치로 해석하지 않는다. 대표 수치와 원본·PDF 해시는 [시각 대조 기록](pr_6759_6768_visual_sweep.md)에 적었다.

추가 WASM canvas 캡처는 페이지 준비 대기 시간 초과로 완료하지 못했다. 이 PR의 시각 근거는 완료된 native SVG의 Chrome raster 대조이며, 이를 Studio canvas 캡처로 바꾸어 설명하지 않는다.

## 원 PR·이슈 처리 범위

원 PR #6760과 #6756에는 중복·용지 밖 출력 축의 검증과 남는 쪽 배분 차이를 함께 기록한다. 승인된 통합 merge와 devel CI 성공 뒤 실제 closing reference 및 auto-close 상태를 확인하며, 현재는 comment·close를 실행하지 않았다.

## 공통 검증과 승인 경계

전체 실행 명령, 첫 실패와 보정 후 결과, lint·Native Skia·WASM·Studio 결과는 [통합 검증 기록](pr_6759_review_impl.md)에 구분했다. 검증 대상은 체리픽 HEAD에 당시 미커밋 메인터너 보정을 더한 작업 트리였으며, 해당 보정은 이후 `902a208b515e83024502f004a2adaf84c33f18de`로 보존했다. 이를 순수 원 head 또는 최종 통합 PR CI 성공으로 대신 기록하지 않는다.

검토 판정은 GitHub approve·merge 권한 행사와 다르다. commit·push·통합 PR 생성은 작업지시자 승인 범위에서 진행한다. 최종 head CI와 시각 판정의 작업지시자 확인·merge 승인은 별도다. 현재 원격 comment·close·merge는 실행하지 않았다.

## Merge 후 댓글 작성 방식

승인된 통합 merge와 실제 devel CI 성공 뒤에만 [후속 처리 절차](../../manual/pr_review/post_merge.md)를 따른다. 원 PR 수용 출처·merge SHA·실제 PR/devel CI를 적고, 같은 merge SHA의 기존 댓글이 있으면 새 댓글 대신 수정한다. UTF-8 `--body-file`로 게시한 뒤 API로 body를 재조회한다.

아래 대표 PNG만 코멘트 이미지로 사용한다. `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr_6759_6768_20260905/<상대 PNG 경로>`를 Markdown 이미지로 넣어 댓글 안에서 직접 표시하고, [시각 대조 기록](pr_6759_6768_visual_sweep.md)과 해당 기준 PDF를 함께 연결한다. 존재하지 않는 merge SHA나 미완료 시나리오를 완료 증적으로 게시하지 않는다.

![#6760 검토 증적 1](../assets/pr_6759_6768_20260905/visual-6756/issue6756/review/review_002.png)

![#6760 검토 증적 2](../assets/pr_6759_6768_20260905/visual-6756/issue6756/review/review_003.png)
