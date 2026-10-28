# PR #7663 메인터너 보정 — 미계산 탭과 저장 줄 캐시

## 독립 근거와 원인

원 head `9afbee93ec46dd6911dc11cad5ee6b524cc9a553`의 자리표 탭 측정 보정을 적용한 뒤,
동일 원본을 실제 한컴 Print로 출력해 비교했다. 메모리에서는 탭이 전진하지만 HWP/HWPX 저장본의
오래된 LineSeg와 미계산 탭 폭이 같이 전달되면 한컴이 탭을 다시 계산하지 않았다.
원문·탭 정의를 바꾸지 않고 줄 캐시만 생략한 대조군은 한컴에서 정상 탭 정지를 사용했다.

## 생산과 소비 경로

`Paragraph::stored_text_partition_is_dirty`와 `tab_ext_is_placeholder`
→ `serializer/line_cache.rs::can_serialize_line_partition`
→ HWP `body_text.rs` 및 HWPX `section.rs`의 줄 분할 직렬화가 같은 판정을 소비한다.
메모리 레이아웃을 고치거나 원본 raw passthrough를 강제로 재직렬화하지 않는다.
실제 저장 탭 확장이 유효한 문단과 무수정 원본은 기존 경로를 유지한다.

원 PR의 `text_measurement.rs`는 추정 run 폭에서도 자리표 순번을 소비한 뒤 TabDef fallback을
쓴다. 문자 위치 계산과 다음 run의 시작이 같은 탭 전진을 사용해야 한다는 계약이다.
문서 ID·좌표 숫자·임의 clamp 분기는 추가하지 않았다.

## 검증

본문·머리말·셀의 HWP/HWPX6개 입력 전쪽 Native/fresh WASM100%, 독립 Print 쪽수 일치와
직접 확대 판독을 확인했다. 원본/PDF SHA와 base/head 페이지별 값은
[공통 시각 증거](../assets/semanticist21_20261015/README.md)에 있다.
각 입력의 탭 앞뒤 글자·뒤 라틴 run·동일 기준선·소유 영역 포함·비겹침을 관계와±2px로 보호한다.
입력은 PR 테스트가 만드는 명시적 계약 문서이며 일반 한컴 생성 원본이라고 주장하지 않는다.

보정 코드 후보는 `ed97e78cb1dbe73e6d0102a809074718e26a2941`이다.
전체 로컬/원격 게이트의 최종 상태는 [개별 review](pr_7663_review.md)에 연결한다.
추가 HWP3 표 날짜행의 기준부터 있던 캐시/클리핑 문제는 이번 보정으로 해결했다고 보고하지 않는다.

## 전체 회귀 후 원본 저장 행의 보존 경계

1차 code `ed97e78cb`에서 전체10,404건을 끝까지 실행했다.10,402PASS/2FAIL/45skip이며,
실패는 convert_verify_corpus_ratchet의 partition13/20이었다. 원본 HWP3 두 문서는 base에서
IR 차이0인데 새 cache 판정이 각각5개 문단·1개 셀 문단의 원본 LineSeg를 생략했다.
미계산 탭 하나만으로 원본 저장 행도 버린 것이 원인이었다.

`Paragraph.layout_space_metrics`는 같은 줄 구성 결과에서 게시하는 재조판 출처다.
이 출처가 없는 정상 원본 행은 보존하고, dirty 행 또는 재생성한 행의 미계산 탭만 외부에서
재계산하게 한다. 문서 이름·숫자·포맷별 허용 목록으로 대상을 좁히지 않았다.

보정 후 명시적 CLI와 test target을 같은 native-skia build graph로 빌드한 뒤 관련6건을 실행했다.
원본 왕복 partition13/20 및 새 본문·머리말·셀 저장 재열기·미계산 탭 cache 검사4건, 모두PASS다.
기존 baseline/EXPECTED_FAILURES 및 test assertion은 수정하지 않았다.
원시 로그는 `cache-provenance-native-cli-and-tests-build-20261016.log`와
`cache-provenance-focused-six-20261016.log`이며 ignored output의 logs에 보존한다.
새 Native/fresh WASM 생산물의 시각 재검증 및 남은 게이트 결과는 최종 개별 review에 연결한다.
