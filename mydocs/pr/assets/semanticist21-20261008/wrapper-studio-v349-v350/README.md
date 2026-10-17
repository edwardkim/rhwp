# 펼친 중첩 표의 실제 Studio 누름틀 동작

- 검증 source commit: `940ea1322617ccff5a7b47d6920ae96e94cfffad`. 이 자산 추가는 Rust 및 Studio 소스를 바꾸지 않는다.
- 입력: [clickhere-wrapper-current-v175.hwpx](../current-print-inputs-v175/clickhere-wrapper-current-v175.hwpx).
- 입력 SHA-256: `17a174162fd3fcd1c51bd04a25a577396c44bd51ad1c8ab001f9184b16dc4de3`.
- 실제 브라우저: Chrome/140.0.7339.16, Linux headless, viewport 1280×1300.
- 조건: CanvasView의 실제 화면, print profile, 2022 호환, 새 탭, 캐시 및 서비스 워커 우회.
- 초기 JS 원본 SHA-256: `691e4fd480cf4e04f9e599ca7c7a4802ad87b2c141fba93bf94111f4144dcb3b`.
- 초기 WASM 응답 SHA-256: `f16609ba3bac35ab0292a0c15c2bc36d8becb562834f98090fb1831cc2d517f9`.

v349는 Canvas2D 요청으로 실행했고, v356에서 요청·실제 선택 backend가 Canvas2D이며
fallbackReason이 null인 상태로 같은 동작을 다시 확인했다(9.595초·exit0).
v350은 요청 및 실제 선택 backend가 CanvasKit인 상태에서 실행했다.
CanvasKit의 fallbackReason은 null이다. 양쪽 모두 논리 누름틀 2개와 깊이 2의 셀 경로를
확인한 뒤 실제 마우스로 앞 누름틀을 눌렀다. 이후 뒤 누름틀만 남았고 페이지 오류는 없었다.
print profile에서는 안내 문자열이 숨겨지므로 스크린샷의 문자열 유무로 클릭 결과를 판단하지 않는다.

Vite 초기 JS 응답은 원본과 다르므로 단순 바이트 비교를 통과했다고 기록하지 않는다.
소스맵을 제거하고 WASM URL에 대한 Vite의 정확한 한 가지 치환만 복원하면 원본 JS와 같다.
응답 원문·소스맵·복원본·동작 전후의 논리 상태는 ignored
`output/pr-review/semanticist21-20261008/studio-wrapper-field-pointer-v349/`와 v350에 보존한다.
초기화 후 별도 fetch 결과를 초기 실행의 증거로 사용하지 않았다.

이 자료는 두 backend의 해당 누름틀 동작 및 실제 화면 증거다. 독립 Print와의 전쪽 일치,
Native Skia, 다른 입력의 clip·폰트·그림 소비 경계 및 최종 PR 수용을 대신하지 않는다.

| PNG | SHA-256 |
| --- | --- |
| [canvas2d-before.png](canvas2d-before.png) | `fda92cf9ae8d846fece0f35e87b3de6cb2bd08a694f20dc0d2dc23397d70a80c` |
| [canvas2d-after-first-field-click.png](canvas2d-after-first-field-click.png) | `83db1e17cbbd09148775019e86e03eea343045e264f9461821a6e0e2dee8da39` |
| [canvaskit-before.png](canvaskit-before.png) | `e584c17082248d65e2b9c0fe7c586ddd1b70ee16cdb3377be10ae02120171b68` |
| [canvaskit-after-first-field-click.png](canvaskit-after-first-field-click.png) | `67c0f619f720c61f64f9061d127857c59001a25405edbeaf785b4f5b0383d64b` |
