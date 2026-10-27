# PR #8027 리뷰 — fix(emf): EMR_POLYGON16·EMR_POLYLINE16 레코드 번호를 바로잡는다

## 최종 판정

**증적 대응 정정 — 대체 입력의 검증 범위로 한정.**

이미 완료된 [통합 PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 merge와 코드 CI 성공 이력은 유지한다. 실제 검증한 입력과 원 PR의 배경 실물 문서를 구분하며 원문 부재만으로 통합을 다시 보류한 것으로 기재하지 않는다.

## 원 PR과 통합 이력

- 원 PR: [#8027](https://github.com/edwardkim/rhwp/pull/8027) / semanticist21.
- 원 contributor head: `1e2532049a260fef9dd814a22d8c1d1253c4b109`.
- 통합 merge: `af6bb05b26887135fb9da0fa4e54ca7205564fca`.
- [통합 당시 리뷰](https://github.com/edwardkim/rhwp/blob/af6bb05b26887135fb9da0fa4e54ca7205564fca/mydocs/pr/archives/pr_8027_review.md)는 과거 기록이며 이 정정의 입력 대응을 대신하지 않는다.

## 주장과 실제 입력의 대응

원 PR의 보고 문서: **재정경제부 고용동향 브리핑 HWPX 3쪽**.

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 유사 문서로 대체했다. 아래 결과는 명시한 대체 입력의 검증이며 원 PR에서 보고한 실물 문서를 직접 검증했다는 뜻이 아니다.**

공식 배포처에서 확보한 대체 문서: [pr8027-official.hwpx](../assets/pr8319_evidence_alignment_20261027/official-inputs/pr8027-official.hwpx), SHA-256 `e75eb152f9d2059c76c83c631acb64883d413da6d491371a667a5c3740ac6499`.

출처: [공식 게시물](https://mofe.go.kr/nw/nes/detailNesDtaView.do?menuNo=4010100&searchBbsId1=MOSFBBS_000000000028&searchNttId1=MOSF_000000000079660). 3쪽 두 차트의 고용률·실업률 및 계절조정 취업자 곡선이 원 PR 캡처와 일치한다.

공식 문서의 내용 대응 확인과 작성자 원본의 바이트 동일성은 구분한다. 새로 찾은 문서를 전 페이지 90% 이상으로 보정하는 것은 작업지시 범위에 포함되지 않는다.

현재 연결한 실행 근거는 파일을 소비하지 않는 단위 계약이다. 함수의 반환·경계 검증과 코드 CI 결과를 기록한다. 다른 파일의 PNG를 그 단위 검사의 실물 문서 증거로 붙이지 않는다.

## 구현과 회귀의 검증 범위

EMF parser의 0x55/0x56/0x57를 각각 POLYBEZIER16/POLYGON16/POLYLINE16로 분리하고, dispatch에서 열린 선과 닫힌 다각형의 record 종류를 서로 바꾸지 않는다. 범주·값을 다시 구성하는 OOXML 차트 로직과 저장된 OLE 미리보기의 EMF 소비 경로를 구분한다. legacy 차트의 물리 배치·축 라벨 보정은 원 record 종류 수정과 별도의 공통 paint 문제로 검토한다.

src/emf/parser/mod.rs의 상수와 record dispatch, src/emf/tests.rs의 16bit record 검사 및 mydocs/pr/assets/legacy_chart_contracts_20261020/README.md와 legacy_chart_physical_layout_20261020/README.md의 독립 Print 기록.


실제 회귀 source:

source/test는 이번 증적 정정에서 변경하지 않았다. [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)의 archive 11,638 PASS, Native 15묶음·124 PASS와 [trailing CI](https://github.com/edwardkim/rhwp/actions/runs/40496300856) 성공은 실제 이력으로 보존한다. 코드 CI를 원 보고 문서의 Print 대조 완료로 확대하지 않는다.

## 기존 댓글 수정

기존 [댓글](https://github.com/edwardkim/rhwp/pull/8027#issuecomment-6440891320)을 직접 수정한다. 추가 정정 댓글을 만들지 않는다. 자료 반영 후 정확한 merge SHA의 URL로 바꾸고 API로 재조회한다.
대체 입력은 이미지 설명에도 파일명·페이지·대체 여부를 밝힌다. 원문 검증으로 오인하게 하는 이미지·수치·완료 주장은 제거한다.

[전체 대응 원장](../assets/pr8319_evidence_alignment_20261027/alignment.json)과 [공식 배포처 탐색·대체 표시](../assets/pr8319_evidence_alignment_20261027/source-identification.md)를 함께 확인한다.
