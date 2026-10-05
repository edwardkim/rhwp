---
kind: report
status: historical
last_verified: 2026-10-02
---

# PR #7508 리뷰 — 첫 문단 클립보드 구조 슬롯 해제

## 최종 판정

승인 — 보류 PR을 제외한 선별 통합 후보의 전체 Rust·필수 lint·fresh WASM 표시 계약을 통과했다.
통합 PR #7511의 code candidate `39f0a2792`와 최종 문서 head `f2a9341f7`의 필수 게이트를 확인했다.
작업지시자의 병합 승인으로 2026-10-02 12:36:38 KST에 devel로 병합했다.
merge SHA: `c77ed685e21de0b2edca918efcec7c2ebbbf2bd5`.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](../pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `b53d3621be516ed8f918b321a0e0b01f09f8ff19`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7508) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36949216335/job/110664169785).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 범위와 소비 경로

#7506 종료 제안. 기능 commit `b53d3621`를 누적 적용했다.
`strip_structural_controls_for_text_clipboard`가 구역/단 정의의 선행 8-slot을 걷고
char_offsets·char_shapes·range tags·markpen·LineSeg start/char_count를 같이 옮긴다.
누름틀 field_ranges는 scalar 축이라 같은 stream shift를 적용하지 않고 필요 시 offsets를 재구성한다.
복사본 → paste_internal의 merge_from → 재조판 → 최종 text layout의 글자 보존을 확인했다.
일반 페이지네이션/renderer 자체는 바꾸지 않는다.

## 검증 결과와 제한

- focused 1/1 PASS: 첫 문단 두 문단 복사, 서식의 ‘가나’ 소속과 전체 표시 글자.
- fresh WASM 실제 브라우저: 마지막 문단의 IR text와 최종 getPageTextLayout가
  둘 다 ‘마바사아가나다라’로 PASS.
- e509 기준 merge simulation clean. 원 exact-head Full CI 성공.
- 모든 혼합 field/남은 컨트롤 조합이 검증됐다는 주장은 하지 않는다. 한컴 픽셀 비교나 Studio UI 검증도 아니다.

## 통합 및 후속 처리

선별 source head `514d4933b`, 검증 checkout `f27661e63`에서 전체 Rust 10,243 PASS / 0 FAIL / 50 skip,
필수 Clippy 3종·workspace build·fmt·manifest 및 fresh WASM 실제 API 3/3를 통과했다.
[선별 후보 최종 결과](../pr_semanticist21_20261002_review_impl.md#선별-후보의-최종-rust-결과)에 source SHA와 명령을 연결한다.
원 head CI는 선별 후보의 GitHub CI를 대신하지 않는다. 선별 코드는 [통합 PR #7511](https://github.com/edwardkim/rhwp/pull/7511)로 게시했다.
[게시 head의 Full CI](https://github.com/edwardkim/rhwp/actions/runs/36955832835)와 CodeQL·Render Diff·Adapter·Proptest는 모두 성공했다.
관련 #7506은 devel 병합 후 자동 종료됐다. 원 PR은 통합 반영을 안내하고 중복 병합하지 않고 닫는다.
기여자의 fork branch는 보존한다. 후속 댓글에서 이 기록의 확정 commit과 merge SHA를 연결한다.

## 2026-10-05 누적 체리픽 적용 이력

기존 #7511 병합 완료 기록은 위에 보존한다. 이번 누적 검토에서 원 PR 후행 검사를 추가로 적용했으며,
이 추가 commit의 최종 누적 검증은 기존 #7511의 성공과 구분한다.

| 원 commit SHA | 상태 | 로컬 적용 SHA |
| --- | --- | --- |
| `b53d3621be516ed8f918b321a0e0b01f09f8ff19` | already-applied | `기존 devel에 포함` |
| `97dd139fcd58361e5ae4834465bf5408b349d00c` | applied | `65e78279a77f352682ee769d35d228a310c9c54a` |

추가 commit은 `strip_section_scoped_controls`의 설명과 한컴 HWPX 클립보드 조각 회귀를 보강한다.
원 저자와 cherry-pick 출처를 보존했다. 최종 후보의 lint·전체 회귀 확인 뒤 해당 범위의 판정을 갱신한다.

## upstream/devel 위 rebase 적용 위치 — 2026-10-05

기준 `c167dc6abbebf69546575e2d16d06223791bab82`. 아래는 현재 이력의 실제 적용 위치이며 위의 이전 검증 SHA는 당시 이력으로 보존한다.

| 원 commit SHA | rebase 전 로컬 SHA | 현재 적용 SHA | 상태 |
| --- | --- | --- | --- |
| `b53d3621be516ed8f918b321a0e0b01f09f8ff19` | `기존 devel 포함` | `기존 devel 포함` | already-applied |
| `97dd139fcd58361e5ae4834465bf5408b349d00c` | `65e78279a77f352682ee769d35d228a310c9c54a` | `7f88ea699b288b5ecd755fe620c3cdb0d3abc0f6` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.
