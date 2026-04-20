# v0.7.3 — 라이브러리 / v0.2.1 — 확장

**라이브러리** (Cargo, @rhwp/core, @rhwp/editor, rhwp-vscode, rhwp-studio): **v0.7.3**
**확장** (rhwp-chrome / Edge / Safari): **v0.2.1**

라이브러리와 확장은 별도 버전 정책으로 운영합니다. 본 릴리스는 두 영역의 변경을 함께 묶었습니다.

---

## 🎯 사용자 직접 영향 (확장 v0.2.1)

### 버그 수정
- **일반 파일 다운로드의 "마지막 저장 위치 기억" 동작 복원** — 확장 활성 시 바탕화면으로 떨어지던 문제 (chrome-fd-001 사용자 보고, [#198](https://github.com/edwardkim/rhwp/issues/198))
- **옵션 페이지 CSP 호환 수정** ([#166](https://github.com/edwardkim/rhwp/issues/166))
- **DEXT5 류 다운로드 핸들러** 에서 빈 뷰어 탭 차단 ([#198](https://github.com/edwardkim/rhwp/issues/198))
- **Windows 한글 파일 경로 처리** 오류 수정 (PR [#152](https://github.com/edwardkim/rhwp/pull/152) by @dreamworker0)
- **모바일 드롭다운 메뉴** 아이콘/라벨 겹침 (PR [#161](https://github.com/edwardkim/rhwp/pull/161) by @seunghan91)
- **썸네일 로딩 스피너 + options CSP** (PR [#168](https://github.com/edwardkim/rhwp/pull/168) by @postmelee)

### 기능 개선
- **HWP 파일 Ctrl+S** 시 같은 파일에 직접 덮어쓰기 — 저장 다이얼로그 매번 안 뜸 (PR [#189](https://github.com/edwardkim/rhwp/pull/189) by @ahnbu)
- **회전된 도형 리사이즈 + Flip 처리** 개선 (PR [#192](https://github.com/edwardkim/rhwp/pull/192) by @bapdodi)
- **HWPX 파일** 열람 시 베타 안내 + 직접 저장 비활성화 — 데이터 손상 방지 ([#196](https://github.com/edwardkim/rhwp/issues/196))
- **HWPX Serializer** Document IR → HWPX 저장 (PR [#170](https://github.com/edwardkim/rhwp/pull/170) by @seunghan91)
- **HWP 그림 효과 (그레이스케일/흑백)** SVG 정확도 개선 (PR [#149](https://github.com/edwardkim/rhwp/pull/149) by @marsimon)
- **HWPX ZIP 압축 한도 + strikeout shape 화이트리스트 + 도형 리사이즈 클램프** (PR [#153](https://github.com/edwardkim/rhwp/pull/153), [#154](https://github.com/edwardkim/rhwp/pull/154), [#163](https://github.com/edwardkim/rhwp/pull/163) by @seunghan91)
- 제품 정보 다이얼로그의 버전 표시 정상화

---

## 🛠 라이브러리 (v0.7.3)

본 릴리스는 다음 npm/마켓 패키지에 동시 배포됩니다:
- `@rhwp/core` — WASM 파서/렌더러
- `@rhwp/editor` — iframe 임베드 에디터
- `rhwp-vscode` — VS Code Marketplace + Open VSX

라이브러리 측 변경은 본 사이클 외부 기여 기반 안정화가 중심.

---

## ⚠️ 알려진 한계

- **HWPX 직접 저장은 비활성화** — HWPX→HWP 완전 변환기 ([#197](https://github.com/edwardkim/rhwp/issues/197)) 완성 시까지. HWPX 파일 열람·편집은 정상이지만 저장은 막혀있습니다. 중요 HWPX 문서는 작업 전 백업해주세요.
- **인쇄 미리보기 창 크기 비정상 확대** — 일부 환경 (Chrome about:blank 줌 메모리). Ctrl+0 으로 리셋 가능 ([#199](https://github.com/edwardkim/rhwp/issues/199)).

---

## 🙏 외부 기여자

이번 릴리스는 다음 분들의 기여로 완성되었습니다:

- [@ahnbu](https://github.com/ahnbu) — PR [#189](https://github.com/edwardkim/rhwp/pull/189)
- [@bapdodi](https://github.com/bapdodi) — PR [#192](https://github.com/edwardkim/rhwp/pull/192)
- [@dreamworker0](https://github.com/dreamworker0) — PR [#152](https://github.com/edwardkim/rhwp/pull/152)
- [@marsimon](https://github.com/marsimon) — PR [#149](https://github.com/edwardkim/rhwp/pull/149)
- [@postmelee](https://github.com/postmelee) — PR [#168](https://github.com/edwardkim/rhwp/pull/168)
- [@seunghan91](https://github.com/seunghan91) — PR [#149](https://github.com/edwardkim/rhwp/pull/149), [#153](https://github.com/edwardkim/rhwp/pull/153), [#154](https://github.com/edwardkim/rhwp/pull/154), [#161](https://github.com/edwardkim/rhwp/pull/161), [#163](https://github.com/edwardkim/rhwp/pull/163), [#170](https://github.com/edwardkim/rhwp/pull/170)

진심으로 감사드립니다 🎉

---

## 📦 다운로드 / 설치

| 채널 | 링크 |
|---|---|
| Chrome Web Store | (배포 진행 중) |
| Microsoft Edge Add-ons | (배포 진행 중) |
| VS Code Marketplace | https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode |
| Open VSX | https://open-vsx.org/extension/edwardkim/rhwp-vscode |
| npm `@rhwp/core` | `npm install @rhwp/core` |
| npm `@rhwp/editor` | `npm install @rhwp/editor` |
| Demo (GitHub Pages) | https://edwardkim.github.io/rhwp/ |

---

## 🔗 관련 자료

- 전체 변경 이력: README 의 "최근 변경" 섹션
- 후속 이슈: [#197](https://github.com/edwardkim/rhwp/issues/197) (HWPX 완전 변환기), [#199](https://github.com/edwardkim/rhwp/issues/199) (인쇄 미리보기 줌)
- 소스 코드: https://github.com/edwardkim/rhwp
- 이슈 보고: https://github.com/edwardkim/rhwp/issues
