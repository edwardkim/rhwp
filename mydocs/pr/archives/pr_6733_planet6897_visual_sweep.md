# PR #6733 사다리 반올림 여유 시각 증적

## 기준

- source head: `13450dc20969ae7a386d4254c323a0ef2dbfca88`
- 통합 cherry-pick: `734528ca3`
- fixture: `samples/issue6718/27469-child-allowance-retroactive-support.hwp`
- fixture SHA-256: `f619b8745d179562755e767307a3728dc7f8952fe70d376fbfe28aaf77ff66d7`
- 출력기: macOS native-skia `rhwp export-png --profile print --page 9`
- 출력: [candidate p10](../assets/pr_6702_6732_planet6897_integration_20260904/visual-6733/candidate-p10/27469-child-allowance-retroactive-support.png),
  794 x 1123, SHA-256 `e76972af16f901044d42bef7f8d6b03cd808edf1ec90827b3942c4a10d085cde`

## 직접 확인 결과

- `rhwp info --json`은 candidate가 fixture를 12 logical pages로 읽는다고 보고했다.
- logical page 10 export에는 footer `- 10 -`이 표시되고 본문이 출력된다.
- 같은 candidate의 focused test는 2쪽·4쪽·10쪽 body-overflow assertion을 모두 통과했다.

## 한계

- 현재 macOS native export에는 fixture의 일부 한글 font glyph 대체가 보인다.
- 따라서 이 PNG는 page 10의 존재와 overflow regression 범위를 보완하는 자료일 뿐, Hancom PDF나
  `rhwp-studio`와의 시각적 동등성·pixel score를 판정하지 않는다.
- 이 증적은 통합 PR의 최신 head CI 또는 merge gate를 대체하지 않는다.
