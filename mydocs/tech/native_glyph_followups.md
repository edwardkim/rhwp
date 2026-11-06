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
