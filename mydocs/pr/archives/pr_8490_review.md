# PR #8490 검토 — fix(section): 구역 정의의 개요 번호 모양 ID를 읽고 바꾼다

## 최종 판정

**메인터너 보정 후 수용 가능 — 검증한 통합 head를 수용·병합했다.** 원 기여 head와 보정은 아래에서 구분한다. 통합 [PR #8640](https://github.com/edwardkim/rhwp/pull/8640)의 최종 head는 `4cf202d2aba44b0cfdd33465b03e37d1826a3fdc`, merge는 `fd598250b7d8abee39eba1b1c68e604a326faa17`다. 필수 CI·작성자 self-review와 작업지시자 승인을 확인했다. [통합 최종 기록](pr_8640_review.md)을 참조한다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자·base | [#8490](https://github.com/edwardkim/rhwp/pull/8490) / semanticist21 / devel |
| 원 repository·branch | `semanticist21/rhwp` / `fix/section-outline-numbering-id` |
| 원 head | `6b86197e31f820c7ba5c098f3bf34d9e50b739e7` |
| 접수 시 devel | `ed057276c1e0e07e743218d95641e015b46b6715` |
| 최종 검증 base | `6e1de406209ba9a58032f6ca6de45b18aa687d1c` |
| reviewer·API 확인 | `edwardkim` / `2026-11-05T01:41:16.808289+00:00` UTC / `requested_reviewers.users` 확인·지정 전후 head 동일 |
| 규모·당시 mergeability | +107 / -1 / 4 files / `MERGEABLE`·`CLEAN` (조회 시점 참고값) |
| 검토 branch | `review/integration-20261105-semanticist21-next10` |

기본 경로는 maintainer 일반이며 접수·로컬 검증·다수 PR·시각 증적·후속 처리를 적용한다. 원 PR head·fork branch를 변경하지 않았다. [전체 범위·CI 근거·충돌 보정안](semanticist21_20261105_next10_review_plan.md)을 따른다.


## 원 기여 commit과 변경 범위

- `442d5833adba05993be7cd5e7e8975d56fdd5d2d` — fix(section): 구역 정의의 개요 번호 모양 ID를 읽고 바꾼다
- `6b86197e31f820c7ba5c098f3bf34d9e50b739e7` — test(section): 개요 번호 ID를 2로 바꿔 저장한 문서의 한/글 인쇄 PDF를 남긴다

제품 변경 경로: `src/document_core/queries/rendering.rs`, `src/wasm_api.rs`.


## 기여의 의미와 메인테이너 보정

원 기여 commit을 수용 후보의 merge 조상으로 그대로 보존했다. u16 overflow·음수·분수·문자열/객체 타입과 없는 ID를 전체 구역 변경 전에 거부한다. TypeScript 선언을 보완했다.

공통 적용 경로: `queries/rendering.rs` 구역 JSON → `object_ops/common.rs`의 엄격한 ID 검증 → 공통 section 갱신/DocInfo dirty → 실제 가) 머리 렌더링과 HWP/HWPX 저장. 검증 실패 전후 전체 구역을 대조한다.

원 PR의 입력 파일이 없어 독립 생성한 공개 입력과 제공 Print를 대응시켰다. 글꼴을 명시하고 전쪽 raw 96.49/97.10%를 확인했다.

## 기존 해결 이력과 CI 처리 판단

실패 check는 발견하지 않았다. 원 PR의 녹색 결과를 새 통합 후보의 통과로 대신하지 않는다.

원 CI의 녹색/실패 여부를 새 누적 후보의 검증으로 대체하지 않았다. 기존 실패 3건은 기여자 재작업 댓글보다 최신 base의 메인테이너 통합·실제 재검증으로 처리한다. workflow·timeout·탐지 기준을 변경하지 않았고 원격 재실행도 하지 않았다. 충돌 보정은 승인된 기존 계약 보존 방침을 따랐다.

## 실제 검증과 입력 동일성

현재 시각 producer는 `d53115fabc92bd392044d9b906c599baddb4368d`, base는 `6e1de406209ba9a58032f6ca6de45b18aa687d1c`다. 제품 source `584c9503baa651c62d0403dad9cb77c5cb9ffde6`의 마지막 계산 API 보정과 source 범위·추가 실행은 [전체 기록](../assets/semanticist21-20261105-next10/validation.md)에 구분한다.

- 집중 보호 범위: 유효 ID 1↔2·가)제목, 잘못된 7종 ID의 전체 구역 원자성, 양쪽 저장 재열기.
- 독립 한컴 Print와 동일 입력의 전 2쪽: Native 최저 **96.49%**, fresh WASM 최저 **97.10%**. 90% 미만·누락·쪽수 불일치·2%p 초과 하락 없음.
- [입력·Print SHA와 전쪽 TSV](../assets/semanticist21-20261105-next10/validation/post8622/summary.json), [문서별 글꼴 공급](../assets/semanticist21-20261105-next10/font-qualification.md), [직접 확대 판독](../assets/semanticist21-20261105-next10/validation/post8622/direct-reading.md), [대표 그림](../assets/semanticist21-20261105-next10/images/post8622/outline-id-review.png).
- 그림의 실제 입력/쪽: **독립 생성 outline-id2.hwpx 1쪽**. [이미지 SHA와 crop](../assets/semanticist21-20261105-next10/validation/post8622/image-index.tsv)을 연결한다.
- 최신 devel에서 반영된 39 PR도 [80상태·406쪽 보존](../assets/semanticist21-20261105-next10/validation/inherited8622/summary.json)과 실제 CLI/전체 회귀를 별도로 확인한다.

최종 필수 Rust·Native·Studio·fresh WASM 검사는 모두 통과했다. 정확한 계수·명령·source와 무시/건너뛴 기존 검사도 통합 검증 문서에 남겼다. contributor CI 주장·합성 계약 테스트와 독립 실물 시각 증거를 구분했다.

## Merge 후 contributor PR comment 계획

아래 게시 계획에 따라 원 PR별로 이 문서와 [후속 초안](../assets/semanticist21-20261105-next10/comments/pr-8490.md)을 확인한 뒤 원 기여의 성과를 먼저 인정하고 메인테이너 보정의 이유·실제 검증 결과를 한국어 존댓말로 적는다.

이미지는 `assets/semanticist21-20261105-next10/images/post8622/outline-id-review.png`의 **독립 생성 outline-id2.hwpx 1쪽**에 대응하는 그림을 사용한다. merge SHA 확정 후 `https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/semanticist21-20261105-next10/images/post8622/outline-id-review.png`로 고정하고, 실제 devel 포함·HTTP image 형식·해시를 확인한 뒤 게시한다. 임시 output이나 다른 PR의 대표 이미지로 대체하지 않는다. 게시 후 API 본문과 실제 화면을 확인한다. 원격 처리 승인을 받았으며 통합 병합과 원 PR의 자동 MERGED를 확인했다. 댓글·동기화·정리는 후속 정본 순서로 실행한다.

현재 base a95d1d5afa와의 [실제 병합 호환](../assets/semanticist21-20261105-next10/current-base-compatibility.md)도 완료했다. 원 검증 code head를 재작성하지 않았고, 이번 전 124쪽씩의 실제 재출력·이번 API·새 글상자 편집 경로를 함께 확인했다.
