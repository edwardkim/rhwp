# Firefox AMO 제출용 메타 (rhwp-firefox v0.2.1)

**제출일 예정**: 2026-04-23
**대상**: addons.mozilla.org (AMO)
**확장명 (manifest id)**: `rhwp-firefox@edwardkim.github.io`
**제출 zip**: `output/amo/rhwp-firefox-0.2.1.zip` (12.4 MB · 52 파일)

---

## 1. 사전 준비 체크리스트

- [x] manifest version: `0.2.1` (Chrome/Edge 와 정합)
- [x] gecko id: `rhwp-firefox@edwardkim.github.io` (2026-04-23: `rhwp@edwardkim.github.io` 는 타인 선점 → 플랫폼명 포함으로 변경)
- [x] strict_min_version: `112.0`
- [x] **data_collection_permissions: required ["none"]** — AMO 필수 (아래 주의 참조)
- [x] **`gecko_android` 키 생략** — Firefox Android 옵트아웃 (아래 "Android 미지원 선언" 참조)
- [x] 클린 재빌드 (`rm -rf dist && npm run build`)
- [x] symlink dereference 검증 (`download-interceptor-common.js` 실체 파일)
- [x] `web-ext lint` errors 0 / notices 0 / warnings 22 (WASM eval / Vite innerHTML / data_collection_permissions gecko 호환 경고 — 모두 AMO 허용 범위)
- [x] zip 패키징
- [ ] AMO Developer Hub 계정 준비
- [ ] 스크린샷 1280x800 (최소 1장)
- [ ] 프로모션 이미지 (선택)
- [ ] PRIVACY URL 활성 확인

### ⚠️ `data_collection_permissions` 와 web-ext lint 의 충돌

2026-04-23 실측:

- `web-ext lint` 는 이 키가 Firefox 140+ 에서만 지원된다는 이유로 **경고 1건** 을 낸다 (`KEY_FIREFOX_UNSUPPORTED_BY_MIN_VERSION`). Android 경고는 gecko_android 옵트아웃으로 해소.
- 그러나 **AMO 서버 검증은 이 키를 필수** 로 요구한다:
  > The "/browser_specific_settings/gecko/data_collection_permissions" property is required for all new Firefox extensions, and will be required for new versions of existing extensions in the future.
- 결론: AMO 서버 요구사항이 우선. 키를 포함하고 lint 경고는 수용.
- 참고: https://mzl.la/firefox-builtin-data-consent

### ⚠️ Android 미지원 선언 (`gecko_android` 키 생략)

**배경**: 2026-04-23 Firefox Android 호환성 감사 결과, rhwp 의 핵심 API 2개가 Android 에서 동작 안 함이 확인됨:

| API | rhwp 사용처 | Android 지원 |
|-----|------------|-------------|
| `browser.downloads.onCreated/onChanged` | 다운로드 가로채기 (핵심 기능) | **Firefox Android v79 에서 제거됨** — 현재 불가 |
| `browser.contextMenus` | 우클릭 "rhwp로 열기" | **지원 안 됨** (MDN BCD: `version_added: false`) |

이 두 기능이 없으면 rhwp 는 사실상 Android 에서 쓸모가 없다.

**공식 옵트아웃 방식**: `browser_specific_settings` 에서 `gecko_android` 키를 **완전히 생략**.

> MDN 공식 문서 기준:
> "To support Firefox for Android without specifying a version range, the `gecko_android` sub-key must be an empty object (`{}`). **Otherwise, the extension is only made available on desktop Firefox.**"

즉, `gecko_android` 를 쓰지 않으면 자동으로 desktop Firefox 전용이 된다.

**시행착오 기록 (2026-04-23)**:
- 1차 시도: `gecko_android.strict_min_version: "999.0"` → AMO 가 "Unknown strict_min_version 999.0 for Firefox 안드로이드" 에러 (유효 버전 목록에 없음).
- 2차 시도 (현재): `gecko_android` 키 자체 삭제 → 공식 권장 방식으로 정착.

**Android 지원 재검토 시점**:
- Mozilla 가 Firefox Android 에서 `downloads` · `contextMenus` API 를 복원할 때
- 또는 rhwp 가 Android 전용 대체 경로 (예: 파일 선택 UI 기반) 를 구현할 때
- 현재는 별도 이슈로 기록만 (AMO 승인 후 고려)

## 2. 등록 메타 (사용자 노출)

### 2.1 이름

| 언어 | 이름 |
|------|------|
| ko (기본) | rhwp — HWP 문서 뷰어 & 에디터 |
| en | rhwp — HWP Document Viewer & Editor |

### 2.2 한 줄 요약 (Summary, 250자 이내)

**한국어**:
```
한글(HWP/HWPX) 문서를 브라우저에서 바로 열고 편집할 수 있는 무료 오픈소스 확장. 파일은 서버로 전송되지 않고 모든 처리는 WebAssembly 로 브라우저 내에서 수행됩니다.
```

**영문**:
```
Open and edit Korean HWP/HWPX documents directly in your browser. Free and open-source. Files are never uploaded — all processing runs locally via WebAssembly.
```

### 2.3 상세 설명 (Description)

**한국어**:
```
rhwp 는 한국에서 가장 널리 쓰이는 HWP / HWPX 문서를 웹브라우저에서 열어보고 편집할 수 있게 해주는 오픈소스 확장입니다. 한컴오피스가 없어도 됩니다.

주요 기능
- 설치 없이 열기 — 확장 한 번 설치하면 HWP / HWPX 파일을 즉시 열람
- 편집 지원 — 텍스트 입력 · 표 편집 · 서식 변경
- 인쇄 — Ctrl+P 로 인쇄 또는 PDF 저장
- 자동 감지 — 웹페이지의 HWP 링크 옆 파란색 H 배지 표시
- 우클릭 메뉴 — HWP 링크 우클릭 → "rhwp 로 열기"

개인정보 보호
- 파일이 서버로 전송되지 않습니다
- 모든 파싱 · 렌더링 · 편집은 브라우저 내 WebAssembly 로 수행됩니다
- 광고 없음 · 추적 없음

저장 안내
- HWP 파일: Ctrl+S 로 같은 파일에 직접 덮어쓰기
- HWPX 파일: 직접 저장은 현재 베타 단계로 비활성화 (다음 업데이트에서 지원 예정)

라이선스: MIT (개인 · 기업 모두 무료)
오픈소스: https://github.com/edwardkim/rhwp
```

**영문**:
```
rhwp is an open-source extension that lets you open and edit HWP / HWPX documents — Korea's most widely used document format — directly in your web browser. No need to install Hancom Office.

Features
- Open without installation — One install lets you view HWP / HWPX files instantly
- Editing — Text input, table editing, formatting
- Printing — Ctrl+P to print or save as PDF
- Auto-detection — A blue H badge appears next to HWP links on web pages
- Right-click menu — Right-click an HWP link → "Open with rhwp"

Privacy
- Files are never uploaded to any server
- All parsing, rendering, and editing run locally via WebAssembly
- No ads, no tracking

Saving
- HWP files: Ctrl+S overwrites the same file directly
- HWPX files: direct save is currently disabled (beta) — coming in a future update

License: MIT (free for personal and commercial use)
Open source: https://github.com/edwardkim/rhwp
```

### 2.4 카테고리

- 1차: **Other** (Productivity 가 AMO 에는 없음, 가장 가까운 분류 선택)
- 또는 **Web Development** (개발자 친화적)

> AMO 카테고리는 Chrome 과 다름. AMO 의 실제 카테고리 목록 확인 후 결정.

### 2.5 태그 (Tags · 최대 10개)

```
hwp, hwpx, hangul, korean, document, viewer, editor, webassembly, korea, hancom
```

## 3. 권한 사용 근거 (Notes for Reviewers / Reviewer Notes)

AMO 는 권한 별 근거를 reviewer notes 에 기재 권장.

```
이 확장은 한국 표준 문서 포맷인 HWP / HWPX 파일을 웹브라우저에서 열람하고 편집할 수 있게 해줍니다. 파일은 서버로 전송되지 않고 모든 처리는 브라우저 내 WebAssembly 로 수행됩니다.

권한 사용 근거:

- activeTab: HWP 파일 다운로드 가로채기 시 현재 탭에서 뷰어 페이지를 엽니다.
- downloads: HWP / HWPX 파일 다운로드를 감지하고 뷰어로 전환합니다 (downloads.onCreated + onChanged).
- contextMenus: HWP 링크 우클릭 시 "rhwp 로 열기" 메뉴를 제공합니다.
- clipboardWrite: 사용자가 편집 중 문서 일부를 복사할 때 사용합니다.
- storage: 사용자 설정 (자동 열기, 배지 표시 등) 을 저장합니다.
- host_permissions <all_urls>: 임의의 웹페이지에서 HWP 링크를 감지하고 배지를 표시하기 위함. 페이지 내용은 분석하지 않으며 HWP 확장자 링크만 식별합니다.

테스트 방법:
1. 확장 설치 후 https://www.assembly.go.kr 등 공공기관 사이트에서 HWP 파일 다운로드를 시도합니다.
2. 다운로드 시 자동으로 새 탭에서 뷰어가 열립니다.
3. 또는 빈 뷰어 탭에 HWP 파일을 드래그해서 열 수 있습니다.

샘플 HWP 파일은 다음 저장소에서 받을 수 있습니다:
https://github.com/edwardkim/rhwp/tree/main/samples

오픈소스: MIT License
저장소: https://github.com/edwardkim/rhwp
```

## 4. 개인정보 처리방침 URL

```
https://github.com/edwardkim/rhwp/blob/main/rhwp-firefox/PRIVACY.md
```

## 5. 홈페이지 / 지원 URL

- Homepage: `https://github.com/edwardkim/rhwp`
- Support URL / Email: `https://github.com/edwardkim/rhwp/issues`

## 6. 스크린샷 (1280x800 권장 · 최소 1장)

준비 필요:

1. **메인 뷰어 화면** — 한컴 샘플 문서 (예: KTX.hwp) 열어서 도형 + 표 + 텍스트가 풍부한 페이지 캡처
2. **편집 모드** — 텍스트 입력 / 표 편집 모습
3. **자동 감지 배지** — 공공기관 게시판에서 HWP 링크 옆 파란 H 배지
4. **우클릭 메뉴** — HWP 링크 우클릭 시 "rhwp 로 열기" 메뉴 노출
5. **인쇄 미리보기** — Ctrl+P 인쇄 다이얼로그

각 캡처:
- Firefox 창에서 1280x800 viewport
- 한국어 UI / 한국어 샘플 문서 우선
- 영문판도 1~2장 (선택)

## 7. 제출 절차

1. https://addons.mozilla.org/developers/ 접속
2. **Submit a New Add-on** 클릭
3. 배포 방식 선택:
   - **On this site** (AMO listed) — 권장. AMO 검색 노출 + 자동 업데이트
   - **On your own** (Self-distribution) — AMO 검색 미노출, 직접 배포
4. zip 업로드: `output/amo/rhwp-firefox-0.2.1.zip`
5. 자동 검증 대기 (수십초)
6. Listing details 입력 (위 2~5절 메타)
7. 스크린샷 업로드
8. **Submit Version for Review**

## 8. 심사

- 일반적으로 1~5 영업일 (수동 검증 대기열 따라 변동)
- AMO 는 자동 + 수동 검증 병행
- 거부 시: 수정 후 새 버전 zip 재제출 (manifest version 동일 불가, PATCH 올림 필요 → 0.2.2)

## 9. 후속 작업

승인 후:
- README.md / README_EN.md 의 "Coming Soon" 에서 "Firefox Add-ons (AMO)" 의 "(준비 중)" 제거 + 실제 링크 추가
- rhwp-firefox/README.md 의 설치 섹션 갱신
- mydocs/orders/yyyymmdd.md 에 심사 통과 기록
- Discussions 또는 README 에 Firefox 지원 안내

거부 시:
- 사유 분석 + 수정
- 매뉴얼 `mydocs/manual/chrome_edge_extension_build_deploy.md` 4.4 절에 거부 사례 보강
