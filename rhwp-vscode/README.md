# HWP Viewer for Visual Studio Code

VSCode에서 HWP/HWPX 문서를 바로 열어볼 수 있는 확장 프로그램입니다.

[rhwp](https://github.com/edwardkim/rhwp) 프로젝트의 WebAssembly 렌더링 엔진을 기반으로, 한컴오피스 한글 문서를 별도 프로그램 없이 VSCode 안에서 확인할 수 있습니다.

## 기능

- HWP/HWPX 파일 클릭 또는 드래그 & 드롭으로 바로 열기
- Canvas2D 호환 기본 렌더링과 후속 CanvasKit 자동 선택을 위한 문서 capability 판정 기반
- 가상 스크롤 (대용량 문서 지원)
- 줌 인/아웃 (Ctrl+마우스 휠 또는 상태 표시줄 버튼)
- 페이지 네비게이션 (상태 표시줄에 현재 쪽 표시)
- 문서 내 이미지 렌더링 (자르기, 테두리선 포함)
- 머리말/꼬리말 내 이미지 및 표 렌더링
- 글상자 내부 표/그림 렌더링
- 분할 표 셀 세로 정렬
- 오픈소스 폰트 폴백 (파일이 서버로 전송되지 않음)
- CanvasKit 폰트 판정은 확장에 포함된 폰트만 사용하며 외부 웹 폰트는 Canvas2D 경로에서만 사용
- **컨텍스트 메뉴** (탐색기/에디터 탭 우클릭):
  - HWP: SVG로 내보내기
  - HWP: 디버그 오버레이 보기 (개발자용)
  - HWP: 문단 덤프 (개발자용)

## 지원 형식

| 확장자 | 설명 |
|--------|------|
| `.hwp` | 한컴오피스 한글 문서 (바이너리) |
| `.hwpx` | 한컴오피스 한글 문서 (OOXML 기반) |

## 사용법

1. 확장을 설치합니다.
2. VSCode에서 `.hwp` 또는 `.hwpx` 파일을 엽니다.
3. 문서가 자동으로 HWP Viewer에서 렌더링됩니다.
4. 스크롤하여 페이지를 탐색합니다.
5. Ctrl+마우스 휠 또는 하단 상태 표시줄의 +/- 버튼으로 줌을 조절합니다.

별도 프로그램 설치나 설정 없이 바로 사용할 수 있습니다.

## 개발자용

소스에서 직접 빌드하려면 WASM 빌드가 선행되어야 합니다 (`pkg/` 디렉토리에 `rhwp_bg.wasm`, `rhwp.js` 필요).

```bash
cd rhwp-vscode
npm install
npm run compile
```

## Third-Party Licenses

이 확장 프로그램은 다음 오픈소스 라이브러리를 사용합니다.

### Rust 크레이트 (WASM 엔진)

| 크레이트 | 라이선스 |
|---------|---------|
| wasm-bindgen | MIT OR Apache-2.0 |
| web-sys / js-sys | MIT OR Apache-2.0 |
| quick-xml | MIT |
| cfb | MIT |
| flate2 | MIT OR Apache-2.0 |
| encoding_rs | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| usvg / svg2pdf | Apache-2.0 OR MIT |
| pdf-writer | MIT OR Apache-2.0 |

전체 목록: [THIRD_PARTY_LICENSES.md](https://github.com/edwardkim/rhwp/blob/main/THIRD_PARTY_LICENSES.md)

### npm 패키지

| 패키지 | 라이선스 |
|--------|---------|
| webpack | MIT |
| typescript | Apache-2.0 |
| ts-loader | MIT |
| copy-webpack-plugin | MIT |
| @noble/hashes | MIT |
| canvaskit-wasm | BSD-3-Clause |

배포 VSIX에는 두 런타임 패키지의 전체 라이선스 고지문이 각각
`dist/licenses/noble-hashes-LICENSE.txt`와
`dist/licenses/canvaskit-wasm-LICENSE.txt`로 포함됩니다.

### 웹 폰트 (렌더링 폴백)

| 폰트 | 라이선스 |
|------|---------|
| Pretendard | SIL Open Font License 1.1 |
| Noto Sans/Serif KR | SIL Open Font License 1.1 |
| 나눔명조 | SIL Open Font License 1.1 |

> 모든 의존성은 MIT 라이선스와 호환됩니다.

## Notice

본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

## Trademark

"한글", "한컴", "HWP", "HWPX"는 주식회사 한글과컴퓨터의 등록 상표입니다.
본 제품은 한글과컴퓨터와 제휴, 후원, 승인 관계가 없는 독립적인 오픈소스 프로젝트입니다.

## 라이선스

MIT License - [LICENSE](LICENSE)
