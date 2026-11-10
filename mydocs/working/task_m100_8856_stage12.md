# #8856 Native 회귀의 기존 CI 진입점 연결

stage11 `82c6f27a5daa31cd61fbcae08e68f53e5a5bb3b8` 다음 단계다. 새 Native 전용
기호 셀 검사는 기본 전체 회귀에서 feature가 꺼지며, CI의 기존 Native 모듈 목록에도
별도 파일이 포함되지 않았다. 로컬 focused PASS만으로 CI 실행을 주장하지 않는다.

검사 본문은 `tests/common/issue_8856_replacement_symbol_cell.rs`로 이동하고 기존
`skia_native_glyph_replay`의 단일 `#[test]`에서 호출한다. suite manifest의 공용 helper
소비 관계를 함께 갱신했다. 같은 검사를 이중 등록하거나 workflow에 개별 호출을 추가하지 않았다.
제품 코드·글꼴·판정 기준은 변경하지 않았다.

최소 글꼴 sandbox에서 기존 정식 Native 진입점을 실행해 실제21PASS/0FAIL을 확인했다.
수정 전 검사 본문과 새 helper의 함수 본문도 동일하다. rustfmt·prepare·fmt를 통과했다.
새 suite 배정의 최소 글꼴 전체 회귀와 lint는 진행 중이므로 이 커밋에서 완료로 적지 않는다.

임시 실행·로그·원본 비교는 ignored `output/issue8856/final-source-82c6f27a/native-ci-entry/`에 둔다.
제출 전 별도로 발견한 report-628 각주 간격 및 WASM sample10 빈 그림 요청의 원인 확인은
아직 완료하지 않았다. 이 CI 수집 변경을 해당 제품 결함의 해결 증거로 사용하지 않는다.
