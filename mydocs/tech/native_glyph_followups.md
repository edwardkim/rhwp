---
kind: reference
status: active
canonical: mydocs/tech/native_glyph_followups.md
last_verified: 2026-11-06
---

# Native Skia 글리프 후속 구현

## P78 — 실제 준비 실패 진단

엄격한 GlyphRun은 contract 검사뿐 아니라 정확한 face 구성, 리소스 준비, leaf별 작업량
제한을 모두 통과해야 TextRun을 대체한다. 이 준비 과정은 `Result`로 실패 원인을 보존한다.
`NativeGlyphPreparationFailure::GlyphRunRejected`는 공개 preflight와 같은
`NativeGlyphRunReplayProof`를 담아 digest/data-ref/face/cluster/placement 문제를 구분한다.
폰트가 없어 실패한 경우 임의의 시스템 family로 exact replay를 대체하지 않는다.

`LayerRenderDiagnostics.native_glyph_rejections`는 Native 빌드에서만 제공되는 부가 관측이다.
리소스 예산 초과, 지원하지 않는 outline payload, 불완전한 variant part도 각각 구분한다.
최대 256개 report와 UTF-8 경계에서 자른 최대 256바이트 식별자만 보관하고 초과 건수는
`native_glyph_rejections_dropped`에 포화 누적한다. 공통 selection reason은 기존 enum을
유지하며 대응되는 font/cluster reason을 재사용한다. 준비 실패가 TextRun을 숨기지 않는다.

정적 picture에는 같은 bounded report와 dropped count를 함께 저장하고 retained-byte
추정에 포함한다. picture hit, glyph cache hit, cache 비활성 경로의 선택·거절 원인이 일치한다.
앞 sibling이 이미 report 상한을 사용해 subtree의 보고서가 잘렸다면 해당 picture를 보존하지
않는다. 그러한 불완전한 진단은 캐시 키 밖의 앞 sibling 상태에 의존하므로, 다음 재생에서 앞
sibling의 대안이 바뀌었을 때 현재 subtree의 보고서를 숨기거나 잘못 누적할 수 있기 때문이다.
기존 자동 수집 Native suite에 실제 fallback pixels와 cold/warm/cache-off proof 회귀를 추가했고,
기존 renderer unit regression은 공개된 준비 API로 entry/byte budget을 확인한다.
Unicode 식별자·report 상한은 public Native renderer의 cold/warm/cache-off case에서 확인한다.
실행 검증은 통합 head에서 수행하며 실물 OS/GPU·한컴 최종 대조는 별도 사용자 검증 범위다.

## P79 — HancomGlyphs의 문자별 글꼴 대체

지정한 Hancom 수식 face가 없거나 해당 문자를 담지 않으면 일반 수식 표시와 같은
문자별 fallback 사다리를 사용한다. 공급된 원본 face와 설치된 원본 face는 기존 우선순위로
찾고, 실제 glyph를 제공할 때 계속 그대로 사용한다. 수식·CJK fallback 및 placeholder/빈 기호
필터를 공통 선택 코드로 재사용하므로 누락된 첫 글꼴이 뒤의 정상 글꼴을 가리지 않는다.

이 경로는 각 문자를 원본 `positions`와 `shears`, layout baseline + `baseline_shift`에
직접 표시한다. fallback의 자연 전진폭으로 원점을 누적하거나 layout 폭을 다시 계산하지 않는다.
기존 regular 원본 face의 hinting/subpixel 설정도 유지한다.

기존 재배포 fixture만 이용해 없는 family, Latin-only face의 한글 누락, 모든 glyph가 있는
원본 face를 합성한다. 기존 Native equation regression case에서 직접 선택한 원본/대체 face의 pixels와 비교해
서로 다른 전진폭에도 위치·baseline·shear가 보존되고 원본 윤곽이 유지되는지 확인한다.
새 글꼴 다운로드는 추가하지 않았다. 이 변경은 #8337의 Native 수식 fallback 경로를 다루며,
실제 문서·최종 한컴 대조와 각 OS 설치 글꼴 조합 확인은 별도 검증 범위다.
