# PR #8889 구현·보정 기록

## 범위와 출처

[최종 self-review](pr_8889_review.md)의 승인 판정·검증 조건을 따른다.
원본 속성의 공통 소비 경로를 단계별 로컬 커밋으로 보정했고 실제 source의 FAIL→PASS,
독립 Print·전쪽 TSV·같은 쪽 PNG는 [최종 증적](../assets/issue8856/final-20261110/README.md)에서 연결한다.
아래는 문서-only rebase 뒤의 실제 commit이며 처음 빌드한 parent와 자산 commit은 producer에 구분했다.

## 실제 적용 커밋

- `aab7bf7e27bbb3ac9c116e7b011d0b6c91538e99` fix: 배분 정렬 말미 인라인 양식의 간격 소유 복구
- `28256790a20e43cbb2ba74cd8eacac00838d31aa` fix: 표 원형 점선의 Print pen과 도색 계약 복구
- `e0528d4d8ce67938348172e86db66202e069c028` fix: 이전 인쇄 세대의 연속 그림자 공급·재생 복구
- `ca9dc87515a34bb50d85c93c37296a2d215b4c9f` fix: 검증된 기호 공급 폭으로 서식 줄 소유 복구
- `a4fd30a482dc27a29f845c83868d23ed1d17c1be` test: 리베이스 반영 qcms의 Mac ARM 검사 조건 보존
- `be5bc811fbd1dca38c5c172413f603028e83c497` fix: 그림 soft-edge 반경의 원본·왕복 저장 소유 복구
- `2515580bd1b4e38a9ce3eefe0f1650d9fe90e51d` fix: 효과 투명도 저장의 암묵 자르기 기준 보존
- `91d97bc3b6fc60ee2d66252d752871d00bfe678d` fix: 명시 대체 기호의 반각 칸과 음수 자간 분리
- `50f339426b27325137d78abb16a545f0036a73db` fix: 그림 효과 반경을 조판과 공통 그리기 트리에 전달
- `7c8f68dd757ddf86a49dce1af8387af5d1ef4475` fix: 그림의 부드러운 가장자리를 공통 paint에서 복원
- `02e5ddd0671bacea7407934e8cf42e33623a4787` fix: 효과 그림의 2024 인쇄 색과 마스크 전달 분리
- `c426a584043c8be96e6c578e77c8063087c9f1a2` fix: 원본 HFT 글머리표의 전각 전진과 뒤 본문 원점 복원
- `85865948c763b87d057941046e306bb2ebdb0439` fix: 캡션 그림 글머리표와 원본 글꼴 공백 전진 복구
- `796d3e55d32a74d183e72c598c6a02aba3fc3b81` test: 기호 셀 회귀를 기존 Native CI 진입점에 연결
- `08ed24fcba19ef8a201bb49a23e9b858bb9e50a8` fix: 참조 번호의 측정과 paint 칸 및 빈 그림 decode 일치
- `4bd40248a4dcbd2e445a5660385818ff9b3e0ea6` fix: 참조 번호의 원본 글꼴 슬롯과 공통 전진 보존
- `e0ce30729e4d87d2a75376d17c6d351a29ef4345` test: 실제 SVG 주석 번호의 분할 출력 조회 보존
- `1081052170207a88eb9e8cd38e4aac7fba328fd9` docs: 8856 최종 전쪽 회귀와 실제 출력 증적 정리
- `b9d22dfbffc4ccf0dab9bcc9078b30e902a06dab` ci: Native Skia 차가운 캐시 검증 시간 예산 보완

## 보정 경계·회귀와 롤백

- 양식: 마지막 inline의 분산 공간 소유를 보존한다. 일반 text/flags8 저장 줄은 반례로 유지한다.
- 표 테두리: 실제 CIRCLE의 pen·채움/외곽 두 번 도색을 표 경계에만 적용한다. 기존 문단 패턴은 유지한다.
- 효과 그림: 읽기·왕복 저장·공통 노드·알파와2024 인쇄 색/마스크 평면을 같은 원본 반경으로 전달한다.
- 기호/글꼴: 표시 윤곽과 소스 전진·음수 자간을 분리하며, 원본 HFT bullet 칸과 뒤 본문 원점을 보호한다.
- 캡션/참조: 원본 그림 마커, Latin 슬롯·상대 크기·장평, 공통 전진 배열을 Native/SVG/Canvas에 전달한다.
- 빈 그림: 실제 비어 있는 unresolved payload만 decode하지 않는다. 외부 원본 그림 공급 후 실제 출력도 검증했다.
- 기존 SVG 검사: 실제 분할 `<text>`를 인접 문자열로 조회한다. 원래 ±2px/내용 보호를 삭제하지 않았다.
- Native CI: 새 회귀를 기존 실행 entry에 수집했다. 별도 worker/중복 호출을 추가하지 않았다.
- CI 시간: timeout30→45분만 보완했다. 되돌릴 경우 이 한 필드만30으로 복원하며 시험/필수 check는 유지한다.

각 원인별 롤백은 해당 공통 경로의 구현과 회귀를 함께 대조한다. 기존 정상 출력이나 독립 기대값을
일괄 갱신하지 않는다. 실제 원본 바이트·저장 계약과 stage별 음성 대조를 유지한다.

## 검증 중 입력·도구 오류의 구분

sample10의 첫 WASM 그림 공급 누락은 제품 실패가 아니며 같은 원본으로 전체 재측정했다.
참조 Studio 첫 진단의 portable metrics guard 누락도 임시 진단을 바로잡고 전체 capture를 다시 완료했다.
브라우저 종료 뒤 updater/crashpad가 잡은 완료 runner pipe만 해제했으며 assertion/exit 판정을 강제하지 않았다.
현재 증적에는 요청 CK와 실제 Canvas2D fallback 및 동일 초기화 hash가 구분돼 있다.

Native 첫 CI는 assertion 실패가 아닌30분 취소였다. 새45분 job에서32분24초·242건 PASS를 확인했다.
구형 TSV의 CRLF 원본은 hash를 보존했다. 전체 PR whitespace 확인은
`git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check upstream/devel...HEAD`
로 수행했다. 문서 후속 PR은 일반 `git diff --check`도 통과해야 한다.

## 옵션 2 후속 처리

원 code head의 Full CI와 일반 병합을 완료했다. review·impl·오늘할일은 원 PR에 trailing하지 않고
별도 문서 PR에만 포함한다. 원 코드·회귀·workflow·golden·기준 TSV는 이 후속 PR에서 바꾸지 않는다.
병합 후 검증 workflow를 재실행하지 않고 duration 운영 workflow의 실제 결과만 확인한다.
