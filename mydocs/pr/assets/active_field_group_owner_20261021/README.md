# 묶기·풀기 뒤 활성 셀 누름틀의 소유 주소

공개 편집 API로 도형 둘 뒤에 1×1 표와 셀 누름틀을 만든 입력이다. 묶기 전·후와
풀기 전·후를 HWP/HWPX로 각각 저장했다. 입력8개는 실제 한컴2020 MCP의 PrintMethod0·
one-up으로 각각 출력했다. 입력·PDF·버전·전처리·job·TSV·실행 명령의 SHA는
[provenance.json](provenance.json)에 있다. 인증 정보·실행 로그·중간 JSON은 포함하지 않는다.

## 저장본 시각 선행 조건

`6fc2b3490d`의 Native079/fresh WASM031에서8조건 각각 전체1쪽의 raw가 모두100%이며
독립 Print 쪽수도1=1이다. [Native TSV](native/)와 [fresh WASM TSV](wasm/) 원문을 보존했다.
묶기 뒤 HWPX의 표·본문·두 도형을 같은 배율의 review/standalone overlay로 직접 판독했다.
위치·내용은 일치하며 얇은 선의 raster 농도 차이는 남는다. raw100%는 RGB 동일이 아니다.

| 실제 출력 | review | overlay |
| --- | --- | --- |
| Native | [review](native_review.png) | [overlay](native_overlay.png) |
| fresh WASM | [review](wasm_review.png) | [overlay](wasm_overlay.png) |

## 편집 상태의 독립 계약

저장본 그림의 통과는 편집 상태의 활성 주소 보존을 입증하지 않는다. `6fc2b3490d`에서
묶기는 표 번호4→3, 풀기는3→4로 바꾸지만 활성 셀 필드는 옛 번호에 남았다. 같은 필드의
재활성화가 false여야 한다는 idempotence와 활성 필드 끝 입력의 값 보존으로 검사한다.
절대 px 좌표·고정 영역·SVG 문자열로 주소 보존을 대신하지 않는다.

Native080의 공개 API 진단에서는 묶기·풀기 뒤 같은 필드 재활성화가 모두false로 유지됐다.
현재 후보의 정식 before/after 검사, 새 제품의 출력·소스 대응과 통합 전체 검증은 이어서
기록한다. 위079/031의 저장본 시각 결과를 최신 편집 동작 실행으로 바꿔 주장하지 않는다.

[공통 주소 분석·보정](../../../working/task_semanticist21_20261021_group_active_field_owner.md),
[#8042 개별 리뷰](../../archives/pr_8042_review.md)를 함께 따른다. 원본 Print는 셀 누름틀의
편집용 안내 표시나 활성 상태를 인쇄 정답으로 삼지 않는다. 형식별 저장 내용과 API 상태를
각각 확인한다. 전체 필수 검증을 완료하기 전에는 PR 제출·승인·통합하지 않는다.

## 정식 회귀 실행

`tests/cases/active_field_group_root_owner.rs`의 새4개는 같은 소스를 strict rustc로
컴파일해 Native079에서4FAIL, Native081에서4PASS를 확인했다. 두 실행의 build는 모두
성공했다. 실패 원인은 바뀐 표 소유 번호를 활성 주소가 따르지 않은 것이다. 같은 주소
설정의 정상 대조, 필드 범위·본문 보존, 필드 안 입력과 양쪽 저장 내용을 확인한다.
실행 원문은 ignored `emphasis-metric-current-failure-focus-1214.json`에 보존했다.
새 fresh의 편집 API 확인과 전쪽 출력·통합 전체 검증은 계속 수행한다.

## 현재 커밋의 fresh WASM 편집 동작 확인

보정 커밋 `ae6d24e744533f87bf39c9ff79e2b6fb4451ecd9`의 최적화 fresh032 빌드와
Studio 패키지 동기화가 성공했다. Native081의 dirty snapshot 소스187개는 이 커밋의
실제 파일 바이트와 모두 같다. snapshot 당시 HEAD와 실제 보정 커밋을 구분해 기록했다.
같은 입력4개를 Node의 실제 `HwpDocument` 공개 API에서 실행해 이전 fresh031의
4FAIL과 현재 fresh032의4PASS를 확인했다. 실패는 같은 활성 셀 주소를 다시 설정할 때
보정이 필요하다는 결과이며, 빌드·초기화 실패가 아니다. 현재 코드는 같은 주소 설정의
idempotence, 표 소유 번호 이동, 지역 필드 정보, 실제 입력과 HWP/HWPX 저장 내용을 보존한다.
원문은 ignored `active-field-wasm-before-after-1223.json`과 같은 번호 로그에 있다.
전체 통합 검증이나 새 제품의 모든 저장본 시각 확인을 이 편집 API 결과로 대신하지 않는다.
