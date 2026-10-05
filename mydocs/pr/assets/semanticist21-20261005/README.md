# semanticist21 누적 검토 증거

19개 원 PR의 리뷰 기록은 `mydocs/pr/archives/pr_<번호>_review.md`에 각각 있다. #7508의 이번 후속 head는 `pr_7508_review_20261005.md`이다. 이 디렉터리는 실행 증거와 진단 재현 스크립트이며 통합 리뷰 문서가 아니다.

- base: `cdba77b609c399fdef26a6c9e637716aa32c2177`
- code candidate: `1d809afe7b965c9ea6137012d59d139d63b038d0`
- 원 PR/적용 SHA/충돌 보정: `application.json`
- 집중 실행: `focused-command.json`을 cargo 인자로 실행. `focused-results.json`은 74건(73 PASS/1 FAIL)을 원 case별로 구분한다.
- Native base 진단: `base-probes.txt`의 2개 테스트는 동일 원본의 표 좌표 및 긴 HTML 문단 길이를 관측한다. 테스트 기대값/golden 변경을 하지 않았다.

## Chromium 진단 재현

저장소 루트에서 fresh WASM과 `rhwp-studio/public`의 해시 일치를 먼저 확인한다. `npm --prefix rhwp-vscode ci --ignore-scripts`로 TypeScript를 설치하고, `npm install --prefix output/pr-review/semanticist21-20261005/browser --no-save --ignore-scripts playwright-core`로 Chromium 제어 의존성을 준비한다. Chromium 실행 경로는 이 서버의 `/snap/bin/chromium`이다.

```bash
python3 -m http.server 18765 --bind 127.0.0.1
# 별도 셸, 저장소 루트
node mydocs/pr/assets/semanticist21-20261005/browser-review.mjs
node mydocs/pr/assets/semanticist21-20261005/kerning-browser.mjs
node mydocs/pr/assets/semanticist21-20261005/outline-runtime.cjs
```

스크립트 출력 경로는 `output/pr-review/semanticist21-20261005/`다. 진단 출력은 공식 Visual Sweep gate나 한컴 PDF의 독립 기준을 대체하지 않는다. 안내문은 사용자 지시에 따라 screen 상태만 검사하고 PDF 비교에서 제외한다.
