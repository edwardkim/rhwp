# rhwp — HWP 문서 뷰어 & 에디터 (Chrome/Edge 확장)

브라우저에서 HWP/HWPX 파일을 바로 열고 편집할 수 있는 확장 프로그램입니다.

## 특징

- **설치 없이 열기** — 확장 설치 한 번이면 HWP 파일을 브라우저에서 바로 열람
- **편집 지원** — 텍스트 입력/수정, 표 편집, 서식 변경
- **인쇄** — Ctrl+P로 인쇄, PDF 저장 가능
- **자동 감지** — 웹페이지의 HWP 링크를 자동 감지하고 아이콘 표시
- **개인정보 보호** — 파일이 서버로 전송되지 않음, 모든 처리는 브라우저 내 WASM
- **무료** — MIT 라이선스, 개인/기업 무료
- **광고 없음**

## 설치

- [Chrome Web Store](#) (준비 중)
- [Microsoft Edge Add-ons](#) (준비 중)

## 사용 방법

### HWP 파일 열기

1. **웹 다운로드**: HWP 파일 다운로드 시 자동으로 뷰어 탭에서 열림
2. **드래그 & 드롭**: 확장 아이콘 클릭 → 빈 뷰어 탭 → 파일 드래그
3. **우클릭 메뉴**: HWP 링크 우클릭 → "rhwp로 열기"
4. **배지 클릭**: 웹페이지의 HWP 링크 옆 파란색 H 배지 클릭

### 인쇄

- 파일 메뉴 → 인쇄 또는 **Ctrl+P**
- 인쇄 미리보기 창에서 [인쇄] 버튼 클릭

### 저장

- **Ctrl+S** 또는 파일 메뉴 → 저장
- **HWP 파일**: 같은 파일에 직접 덮어쓰기 (저장 다이얼로그 없음, v0.2.1 부터)
- **HWPX 파일**: 직접 저장은 현재 베타 단계로 비활성화 (다음 업데이트에서 지원 예정 — [#197](https://github.com/edwardkim/rhwp/issues/197))

## 웹사이트 개발자

공공 웹사이트에 `data-hwp-*` 속성을 추가하면 사용자 경험이 향상됩니다.
자세한 내용은 [개발자 가이드](DEVELOPER_GUIDE.md)를 참조하세요.

```html
<a href="/files/공문.hwp" data-hwp="true" data-hwp-title="공문" data-hwp-pages="5">
  공문.hwp
</a>
```

## 빌드

```bash
cd rhwp-chrome
npm install
npm run build
```

빌드 결과물은 `dist/` 폴더에 생성됩니다.

### 개발 모드 설치

1. `chrome://extensions` (또는 `edge://extensions`)
2. 개발자 모드 활성화
3. "압축 해제된 확장 프로그램을 로드합니다" → `rhwp-chrome/dist/` 선택

## 변경 이력

### v0.2.1 (2026-04-19)

**버그 수정**
- 일반 파일 다운로드 시 마지막 저장 위치가 기억되지 않던 문제 수정 ([#198](https://github.com/edwardkim/rhwp/issues/198))
- 옵션 페이지의 스크립트가 동작하지 않던 CSP 문제 수정 ([#166](https://github.com/edwardkim/rhwp/issues/166))
- 일부 관공서 사이트 (DEXT5 류) 다운로드 시 빈 뷰어 탭이 뜨던 문제 차단
- Windows 환경의 CFB 경로 구분자 오류 수정 (외부 기여 by [@dreamworker0](https://github.com/dreamworker0) — PR [#152](https://github.com/edwardkim/rhwp/pull/152))
- 모바일 드롭다운 메뉴 아이콘/라벨 겹침 수정 (외부 기여 by [@seunghan91](https://github.com/seunghan91) — PR [#161](https://github.com/edwardkim/rhwp/pull/161))
- 썸네일 로딩 스피너 정리 + options CSP 호환 (외부 기여 by [@postmelee](https://github.com/postmelee) — PR [#168](https://github.com/edwardkim/rhwp/pull/168))

**기능 개선**
- HWP 파일을 `Ctrl+S` 로 저장 시 같은 파일에 직접 덮어쓰기 — 저장 다이얼로그가 매번 뜨지 않음 (외부 기여 by [@ahnbu](https://github.com/ahnbu) — PR [#189](https://github.com/edwardkim/rhwp/pull/189))
- HWPX 파일 열람 시 베타 안내 표시 + 저장 비활성화 (데이터 손상 방지)
- 회전된 도형 리사이즈 커서 개선 + Flip 처리 (외부 기여 by [@bapdodi](https://github.com/bapdodi) — PR [#192](https://github.com/edwardkim/rhwp/pull/192))
- HWP 그림 효과(그레이스케일/흑백) SVG 반영 (외부 기여 by [@marsimon](https://github.com/marsimon) — PR [#149](https://github.com/edwardkim/rhwp/pull/149))
- HWPX Serializer 구현 — Document IR → HWPX 저장 (외부 기여 by [@seunghan91](https://github.com/seunghan91) — PR [#170](https://github.com/edwardkim/rhwp/pull/170))
- HWPX ZIP 압축 한도 + strikeout shape 화이트리스트 + 도형 리사이즈 클램프 (외부 기여 by [@seunghan91](https://github.com/seunghan91) — PR [#153](https://github.com/edwardkim/rhwp/pull/153), [#154](https://github.com/edwardkim/rhwp/pull/154), [#163](https://github.com/edwardkim/rhwp/pull/163))
- 제품 정보 다이얼로그의 버전 표시 정상화

**기여자 감사**
이번 배포 주기에 기여해주신 분들: [@ahnbu](https://github.com/ahnbu), [@bapdodi](https://github.com/bapdodi), [@dreamworker0](https://github.com/dreamworker0), [@marsimon](https://github.com/marsimon), [@postmelee](https://github.com/postmelee), [@seunghan91](https://github.com/seunghan91)

### v0.1.1 (이전)

- 초기 공개 베타

## 향후 예정

- **HWPX 직접 저장 지원** ([#197](https://github.com/edwardkim/rhwp/issues/197)) — HWPX → HWP 완전 변환기 구현
- **What's New 알림** — 업데이트 시 변경사항을 사용자에게 자동 안내
- **추가 다운로드 핸들러 패턴 지원** — 사용자 보고 기반으로 인터셉터 정확도 향상

## 라이선스

MIT License — Copyright (c) 2026 Edward Kim

## 개인정보 처리방침

[PRIVACY.md](PRIVACY.md) 참조
