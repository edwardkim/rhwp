# 단계 5 — Studio 저장·재열기와 브라우저 PDF 링크 검증

- Issue: [#6963](https://github.com/edwardkim/rhwp/issues/6963)
- 실행일: 2026-09-10
- 기준: 단계 4 커밋 `daabac0a8`
- 상태: 계획한 Chrome 전체 여정 통과. PR 통합 검증·원격 게시 전.

## 결과

Studio에서 링크를 만든 뒤 실제 저장 명령으로 생성한 HWP/HWPX를 다시 열고,
그 문서의 PDF 저장 경로가 생성한 인쇄 DOM을 Chromium PDF 엔진에 전달했다.
두 형식 모두 표시 글자 `한컴 링크 😀`, 원래 한글 URL, 필드 ID·범위가 같았다.
최종 PDF에도 `/Link`·`/URI`와 글자에 대응하는 영역이 남았다.

이번 환경은 **Chrome 152.0.7977.83, macOS, headless**다. SVG 안의 투명 `<a>`가
Chromium PDF의 주석으로 변환되므로 HTML 링크를 덧씌우는 제품 코드 변경은 필요하지 않았다.
이 단계에서는 회귀 테스트, 독립 PDF 검사 도구와 증적만 추가했다.

| 산출 PDF | 페이지 | 링크 주석 | 인쇄 DOM 대비 영역 최대 차이 |
| --- | ---: | ---: | ---: |
| Studio HWP 저장·재열기 | 1 | 1 | 0.300pt |
| Studio HWPX 저장·재열기 | 1 | 1 | 0.300pt |
| 긴 링크의 자동 줄·페이지 분할 | 2 | 74 | 0.349pt |
| 한컴 Textmail HWP | 1 | 1 | 0.357pt |
| 한컴 LH HWPX | 29 | 4 | 0.381pt |

총 81개 주석의 URI·페이지·영역을 pypdf로 독립 검사했다. 비교 대상 DOM은 실제 인쇄
surface에서 캡처했으며, CSS px→PDF pt(72/96)와 Y축 변환을 적용했다. Chromium의
클릭 영역 양자화를 고려한 허용 오차는 1pt이고, 관측 최대는 0.381pt 미만이다.
HWP와 HWPX 재열기 후 PDF의 주석 목록은 완전히 같다.

## 실제 저장과 뷰어 클릭의 경계

`rhwp-studio/e2e/hyperlink-pdf-issue6963.test.mjs`는 저장소의 Puppeteer harness를 사용해
별도 테스트 Chrome을 실행한다. 도구 버튼과 링크 모달, 파일 메뉴, 저장 이름 확인,
실제 저장 명령·직렬화, 숨김 파일 input을 통한 재열기를 실행한다. OS 파일 picker의
핸들만 테스트 핸들로 대체하여 `write(blob)`에 전달된 바이트를 디스크에 그대로 쓴다.
HWP CFB/HWPX ZIP magic과 재열기 후 필드 내용, 저장 뒤 clean 상태도 검사한다.

PDF는 native 인쇄창 대신 `print()` 시점의 **실제 iframe 문서 전체**를 캡처하고 별도
페이지에서 Chromium `printToPDF`를 호출한다. 운영체제의 인쇄 대상 선택·저장창 자체를
자동 조작한 증거는 아니다. 이는 기존 #3126 인쇄 회귀 테스트와 같은 방식이다.

이렇게 생성한 `studio-hwp.pdf`를 **Chrome 내장 PDF 뷰어**에서 열고 실제 글자 위치
`(390, 142)`를 마우스로 클릭했다. 800×600 viewport의 고정 fixture에서 사용하는 좌표이며
[클릭 전 화면](assets/issue6963/stage5/chrome-pdf-viewer.png)으로 위치를 확인했다.
URL 이동 결과는 다음과 같았다.

```text
https://example.com/%ED%95%9C%EA%B8%80?q=1&lang=ko#%EB%B6%80%EB%B6%84
```

외부 서버에는 요청하지 않고 해당 navigation request를 테스트 HTML 응답으로 대체했다.
뷰어의 실제 hit testing과 이동 주소를 검증했으며 목적지 서버의 응답은 검증 대상이 아니다.

## 한컴 기준과 시각 비교

- Textmail의 `http://www.hancom.co.kr`은 Chrome PDF에서 끝에 `/`가 붙는다.
  루트 경로 정규화만 비교에서 허용하며 query·fragment나 잘못된 문자를 삭제하지 않는다.
- LH p2·p11의 `https://apply.lh.or.kr/`과 p8의
  `https://apply.lh.or.kr/LH/index.html#MN::CLCC_MN_0010:`을 원본 PDF와 대조했다.
  fragment의 `#`, `::`, 마지막 `:`가 그대로 유지된다.
- LH p29의 원본 필드에는 `http://www.lh.or.kr)`가 들어 있고 현재 엔진/Chrome 출력에는
  이 주소의 주석도 생긴다. 한컴 PDF의 이 항목에는 실행 URI가 없어 단계 1 계약처럼
  정상 주소 일치의 성공 기준에서 제외한다. 표의 4개는 출력된 전체 주석 수다.
- 한컴 PDF와의 레이아웃 차이는 단계 3과 동일하게 별도 기록했다. 대표적으로 Textmail의
  X 차이 약 6.5pt, LH p8의 Y 차이 약 29pt가 남아 있다. 위 표의 0.381pt는 **자체 인쇄
  DOM과 주석의 일치**이며 한컴과 문서 레이아웃이 일치한다는 뜻이 아니다.
- 새 문서와 Textmail에서 링크 group을 제거한 별도 PDF를 생성했다. 제거본의 주석은
  0개이며, Poppler 144dpi 이미지 비교는 두 경우 모두 **변경 픽셀 0**이다.
- Textmail p1, LH p8, 긴 링크 p2의 렌더링을
  [페이지 비교 이미지](assets/issue6963/stage5/pdf-pages.png)로 확인했다.

기계 증적:
[브라우저 저장·재열기·클릭](assets/issue6963/stage5/browser-evidence.json),
[PDF 주석·한컴 geometry 비교](assets/issue6963/stage5/pdf-validation.json),
[픽셀 비교](assets/issue6963/stage5/visual-validation.json).

## 재현

단계 4에서 빌드한 실제 WASM을 사용했다. 새 checkout에서는 먼저
[단계 4의 빌드 절차](task_m100_6963_stage4.md)를 따른다.

```bash
# source worktree/rhwp-studio에서 Vite 실행
npm run dev -- --host 127.0.0.1 --port 7763
# 별도 터미널, full Chrome과 pypdf가 설치된 Python 지정
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
VITE_URL=http://127.0.0.1:7763 PYTHON=/path/to/python3 \
node e2e/hyperlink-pdf-issue6963.test.mjs --mode=headless
# 저장소 루트에서 기존 산출만 독립 재검사
/path/to/python3 tools/verify_studio_hyperlink_pdf.py output/pdf/issue6963-stage5
node --check rhwp-studio/e2e/hyperlink-pdf-issue6963.test.mjs
python3 scripts/check_e2e_manifest.py
git diff --check
```

테스트 출력은 `output/pdf/issue6963-stage5/`에 PDF·원본 인쇄 HTML·HWP·HWPX·JSON으로 남는다.
`*-without-links.pdf`와 대응 PDF를 `pdftoppm -r 144 -singlefile -png`로 렌더한 뒤
Pillow `ImageChops.difference(a, b).getbbox() is None`으로 픽셀 비교를 재현할 수 있다.
이번 최종 E2E·독립 PDF 검사·JS/Python 구문 검사·manifest·diff 검사는 모두 통과했다.
Rust나 제품 TypeScript는 이번 단계에서 변경하지 않아 직전 단계의 전체 Node/TS 및
native/WASM library lint를 다시 실행하지 않았다.

## 남은 통합 절차와 지원 범위

계획한 1~5단계의 기본 HTTP/HTTPS 편집→저장→PDF 경로를 로컬에서 검증했다.
PR 준비 단계의 전체 Rust lint 묶음, release/Native Skia/WASM 및 해당 통합 게이트,
원격 push·PR·merge는 아직 실행하지 않았다. 이번 WASM은 단계 4의 `--no-opt` 진단
빌드이며 최적화 배포 빌드의 결과로 확대하지 않는다.

Safari·Firefox·Edge 및 운영체제 인쇄창은 이번 브라우저 검증 범위가 아니다.
편집기 내부 URL 클릭 이동, 신규 mailto·파일·내부 책갈피 UI, 회전/세로 링크 등
[기존 단계의 지원 경계](task_m100_6963_stage4.md)는 그대로다. bare `createEmpty()`의
최소 IR 저장 문제도 미해결이며 실제 Studio의 `createBlankDocument()` 경로와 구분한다.
원래 checkout의 사용자 변경 `samples/exam_eng.pdf`는 수정하지 않았다.
