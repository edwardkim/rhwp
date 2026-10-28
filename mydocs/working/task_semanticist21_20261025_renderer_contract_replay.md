# CanvasKit 재생의 기존 정적 계약 검사 보정

## 원 해결과 실패 원인

기존 `c7be19d7d0`의 positioned text 검사는 원문 표시 문자열의 상한과 직렬화된
전진값 소비를 검사한다. `8a1115103130e58bb8ac9be5cb3a5232c2df19f6`은 실제 선택
글꼴의 가시 PUA를 재생하기 위해 `replayPositions`와 `glyphReplayText`를 로컬 뷰로
교체하도록 `const`를 `let`으로 변경했다. paint op의 원문·주소·전진은 바꾸지 않는다.

현재 실제 구현은 상한 검사 → 유한 위치 검증 → `compatibilityGlyphView`의 원본
위치 선택 → glyph ID 조회 → `runPositions` → `drawCachedGlyphs` 순서로 소비한다.
기존 검사의 `const` 두 개가 실제 구현과 달라 `logs/handbook-node-contracts-3795.log`에서
실패했다. 런타임 전진이나 출력 허용치를 완화하는 변경이 아니다.

## 변경과 검증

기존 정규식의 선언 두 개만 현재의 `let`으로 맞춘다. 표시 상한·위치 출처·glyph 조회·
최종 전진값 소비에 대한 나머지 검사는 유지한다. 기존 대형 검사 파일의 두 토큰 수정으로
범위를 제한하며 이 작업에서 전체 파일을 재구성하지 않는다. 전체 renderer contract의
실행과 실제 compatibility glyph 뷰의 경계·위치 불변식을 함께 확인한다.

로그는 `output/pr-review/semanticist21-20261019/logs/renderer-contract-replay-3810.log`에
기록한다. 검사 종료 전에는 통과로 판정하지 않는다.

## 실제 실행 결과

- 전체 `node --test rhwp-studio/e2e/renderer-contract.test.mjs`: 종료 코드 0,
  `renderer backend contract guard passed`. 기존 런타임 assertions도 끝까지 실행했다.
- `logs/compatibility-replay-contract-3811.log`: 실제 TypeScript helper를 호출해
  가시 PUA 선택의 원래 시작점·최종 전진 보존, 입력 불변, 미지원 글리프의 원문 유지,
  잘못된 위치 길이·겹친 후보 거부를 확인했다. 독립 Print 일치의 대용으로 보고하지 않는다.
- 렌더러 구현·paint 데이터·PNG baseline·수치 허용 오차는 변경하지 않았다.
