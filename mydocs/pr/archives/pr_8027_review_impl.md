# PR #8027 구현·검증 기록 정정

## 현재 판정

**증적 대응 정정 — 원문 미첨부·단위 계약의 검증 범위로 한정.**

**원 PR에 검증용 HWP/HWPX 원본이 첨부되어 있지 않아 현재 실행 근거는 파일을 소비하지 않는 단위 계약에 한정한다. 유사 문서로 대체한 시각 증거가 있는 것으로 표시하지 않으며 배경 실물 문서의 직접 검증을 주장하지 않는다.**

## 구현과 과거 실제 CI

EMF parser의 0x55/0x56/0x57를 각각 POLYBEZIER16/POLYGON16/POLYLINE16로 분리하고, dispatch에서 열린 선과 닫힌 다각형의 record 종류를 서로 바꾸지 않는다. 범주·값을 다시 구성하는 OOXML 차트 로직과 저장된 OLE 미리보기의 EMF 소비 경로를 구분한다. legacy 차트의 물리 배치·축 라벨 보정은 원 record 종류 수정과 별도의 공통 paint 문제로 검토한다.

src/emf/parser/mod.rs의 상수와 record dispatch, src/emf/tests.rs의 16bit record 검사 및 mydocs/pr/assets/legacy_chart_contracts_20261020/README.md와 legacy_chart_physical_layout_20261020/README.md의 독립 Print 기록.


통합 merge `af6bb05b26887135fb9da0fa4e54ca7205564fca`와 [Full code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)는 완료 이력이다. 엔진·테스트·baseline은 이번 변경에서 수정하지 않았다.

## 입력·페이지·대표 증거

[개별 리뷰](pr_8027_review.md)에 실제 검증한 파일·SHA·Print·페이지·TSV와 대표 이미지를 기록했다. 대체 입력은 원 보고 실물 문서의 직접 검증으로 표현하지 않는다. 공식 배포 문서의 내용 대응 확인도 작성자의 원본 바이트 동일성을 뜻하지 않는다.

파일을 소비하지 않는 기존 단위 계약만 검증했다. 실물 문서의 시각 완료 또는 대체 문서의 Visual Sweep 완료를 주장하지 않는다.

## 원격 반영

새 댓글을 추가하지 않고 기존 댓글 ID `6440891320`를 수정한다. 증적 PR의 실제 반영 뒤 merge SHA로 URL을 고정하고 게시 내용을 재조회한다.
