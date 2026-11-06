# 미분류 HWP5 글꼴 선언과 실제 프로그램 조회의 계약

## 선행 해결과 실패 판정

[PR #8397](https://github.com/edwardkim/rhwp/pull/8397)의
`9b0a86aabd141b20f9694276f8b2543dc606d1d6`은 HWP5 FACE_NAME 하위 비트가
TrueType 선언이 아니라는 계약을 검사했다. 직전 테스트 파일 변경은
`5551091085418c17d8321c0fee2626365911481b`(2026-11-04 12:13:59+09:00)다.

현재 통합의 메인터너 보정 `354f88e4eb07287b9b52de462206bd5c5c4ff65c`은
원본과 같은 이름의 실제 TrueType을 조회할 수 있게 했다. 메트릭 표가 없는 미분류
원본만 대상이며, 대체 family나 HFT 선언에는 이 권한을 빌려주지 않는다.
독립 2020 Print에 서울한강체·서울남산체 TrueType이 들어 있는 표지판 관리대장에서
Native 100.00%를 얻었고, 원 프로그램을 공급하지 않는 상태는 기존 전진을 유지한다.
이후 변경된 주소 검증·등록 상한·U+3007 보정은 이 요청 판정을 수정하지 않았다.

R14 집중 검사 36건 중 `unassigned_hwp5_font_bits_do_not_create_truetype_requests`
한 건이 실패했다. 같은 최소 글꼴 환경에서 단독 재실행도 실패했다. 기존 검사는
미분류 비트의 TTF 승격과 같은 이름의 실제 TTF 조회를 모두 금지하는 하나의 assertion을
사용했다. 현재 공통 요청 계약과 맞지 않는 것은 조회 금지 부분이다.

## 유지한 보호와 실제 확인

제품의 `try_resolve_styles_for_document`는 HWP5이고 HWPX 계보가 없으면 여전히
`font_families_source_ttf`를 모두 비운다. `requests_original_truetype`는 이 선언과
별도로 원본 언어 슬롯·최종 이름의 동일성과 기존 메트릭의 부재를 확인한다.
공급된 프로그램도 기존 TrueType/CFF 검사 뒤에만 전진을 제공한다.

검사를 `unassigned_hwp5_font_bits_allow_source_probe_without_truetype_declaration`로
바꾸어 공개 DocumentCore가 발행하는 실제 요청의 원본 이름·TrueType 프로그램 조건과
공급 전 RenderTree 본문 A/가의 `source_font_is_ttf=false`를 함께 검사한다.
조회와 렌더 트리 생성 전후 원본 Document 전체의 동일성도 확인한다. 기존의
HWPX 계보/형식에서 명시적 TTF를 요청하는 대조군과 HFT 선언에서 TTF를 요청하지 않는
반례는 유지했다. 설치 글꼴이나 실제 페이지 좌표에 의존하지 않는 API 상태 계약이다.

최소 공개 글꼴 5개 환경·공유 `target/pr-review`·빌드 잡 8개·검사 스레드 12개에서
R15 집중 검사 **36건 모두 통과**, 실패 0·exit 0을 확인했다. 새 fontName 오류
원자성 검사 5건과 기존 본문/셀 서식·글꼴 세션 계약을 포함한다. 필터 제외 1268건을
통과 건수에 합산하지 않는다. 최초 R14 실패 로그를 덮어쓰지 않았다.

실제 로그는 ignored output의 `common-fixes-r14-native-focused-tests.log`,
`native-font-unassigned-isolated-r14.log`, `common-fixes-r15-native-focused-tests.log`다.
이 결과로 fresh WASM 공개 API나 통합 최종 회귀 완료를 대신하지 않는다.
