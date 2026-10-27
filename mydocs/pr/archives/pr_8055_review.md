# PR #8055 리뷰 — fix(typeset): 쪽을 넘은 위아래 배치 표 조각 뒤 첫 줄의 간격을 한 번만 둔다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8055](https://github.com/edwardkim/rhwp/pull/8055) / semanticist21.
- 원 contributor head: `b5053178fc789e68bf232a3b776edc2e855ed55b`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8055_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **교육부 데이터로 읽는 우리 교육 HWPX**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8055-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8055-official.hwpx), SHA-256 `adb76fe70975d3f8dae20120d1ea27434f24aa69ebd7e04427b80e547bb78828`.

출처: [공식 게시물](https://www.moe.go.kr/boardCnts/viewRenew.do?boardID=294&boardSeq=107440&lev=0&m=020402&opType=N&page=1&s=moe&statusYN=W). 2026-10-18 발간, OECD 교육지표 부제, 일정 1~16번 목록과 2쪽 담당부서·기관 표가 원 PR 전체 캡처와 일치한다. 10쪽 별첨 연구책자는 다른 첨부로 사용하지 않는다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

- 실제 검증 입력: [public-rowbreak-normalized-2020.hwp](../assets/float_fragment_frames_20261020/inputs/hwp/public-rowbreak-normalized-2020.hwp).
- 입력 SHA-256: `5c6e9dcc37c42de5f2d4c9972eb339e476672ac60da02135e7ec2e483d5ed34d`.
- 독립 Print PDF: [public-rowbreak-normalized-2020-2020.pdf](../assets/float_fragment_frames_20261020/pdf/hwp/public-rowbreak-normalized-2020-2020.pdf).
- PDF SHA-256: `93cc27c42b7db35f13e8925c651c883e90b390e68bbf068ad4865e2437bec22f`.
- 대표 대응: 이 입력의 독립 PDF 2쪽 ↔ RHWP 2쪽.

| 출력 | 전체 쪽 | 전쪽 최저 raw | 대표 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | ---: | --- | --- |
| native | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8055/native/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8055/native/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8055/native/overlay.png) |
| wasm | 2 | 100.00000% | 100.0% | [원문](../assets/pr8319_evidence_alignment_20261027/pr_8055/wasm/silhouette.tsv) | [review](../assets/pr8319_evidence_alignment_20261027/pr_8055/wasm/review.png) / [overlay](../assets/pr8319_evidence_alignment_20261027/pr_8055/wasm/overlay.png) |

표의 수치와 PNG는 위 실제 검증 입력의 결과다. 공식 사이트에서 새로 찾은 별도 문서의 수치로 바꾸어 읽지 않는다. 같은 입력·Print·페이지의 Native Skia와 기존 최적화 web package의 Canvas2D 출력을 새로 캡처했다. 엔진은 통합 merge와 동일하며 새 WASM 빌드를 수행한 것으로 기록하지 않는다.

## 구현과 회귀의 검증 범위

분할 표의 마지막 조각 뒤 spacing-before·outer-bottom·host closing band를 중복 소비하지 않는다. 원 합성 NONE 입력을 임의로 RowBreak라 가정하지 않고 정상 한컴 저장본 public-rowbreak-normalized-2020.hwp와 독립 50/10행 두 쪽 Print를 사용한다. 현재 검사는 마지막 조각 뒤 문단의 실제 gap이 원본 bottom margin 범위 안에 있고 다음 문단 내용이 보존되는지 확인한다. 과거 29px 차이를 허용 오차로 덮지 않는다.


실제 회귀 source:
- [tests/cases/split_float_successor_spacing_before.rs](../../../tests/cases/split_float_successor_spacing_before.rs).

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8055#issuecomment-6440922116)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
