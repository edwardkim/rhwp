# 리베이스 후 CanvasKit 합성 계약

검증 source `a2c6d75ec81d9c155dbfe2e58ba814a53c5a33a7`, 새 base `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
후속 문서 변경은 검증한 제품·테스트 함수 본문을 바꾸지 않는다.

새 base에서 바뀐 Studio 소비 경로 `rhwp-studio/src/view/canvaskit-renderer.ts`와
`rhwp-studio/tests/canvaskit-control-ink.test.ts`의 실제 메서드를 실행했다.

```bash
cd rhwp-studio
node --test tests/canvaskit-control-ink.test.ts
```

- 실제 결과: **21 PASS·0 FAIL·0 SKIP·exit 0**.
- 시작 UTC: `2026-11-05T01:21:46.534477+00:00`.
- 종료 UTC: `2026-11-05T01:21:50.477907+00:00`.
- 원시 로그 SHA-256: `8428a1d4f46221d4b8ff86ad5bb19b9de72d0f58c448a7a71f0b6fea6ac34894`.
- 로컬 로그: `output/pr-review/planet6897-20261103/logs/rebase8609-canvaskit-control-ink.log`.

탭 제어문자의 잉크 생략과 소스 전진·후속 글자 원점 보존, 줄 크기에 맞는
밑줄, 공통 장식선 geometry·위상과 유효한 paint 값의 소비를 확인했다.
검증은 Vite로 실제 renderer 모듈을 읽고 메서드에 기록용 CanvasKit 대상을 공급하는
합성 계약이다. 실제 한컴 문서의 CanvasKit 화면 캡처·전쪽 시각 검증·개선율로
확대하지 않는다. 실제 Native/fresh WASM 출력·쪽수·독립 Print 대조는 별도
[통합 검증 기록](README.md)의 해당 실행으로 확인한다.
