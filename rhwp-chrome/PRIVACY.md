# 개인정보 처리방침 / Privacy Policy

## 한국어

**rhwp - HWP 문서 뷰어 & 에디터** 확장 프로그램은 사용자의 개인정보를 수집, 저장, 전송하지 않습니다.

### 데이터 처리 원칙

- **파일은 서버로 전송되지 않습니다.** 모든 HWP/HWPX 파일 처리는 사용자의 브라우저 내에서 WebAssembly(WASM)로 수행됩니다.
- **개인정보를 수집하지 않습니다.** 사용자의 이름, 이메일, 위치, 브라우징 기록 등 어떠한 개인정보도 수집하지 않습니다.
- **외부 서버와 통신하지 않습니다.** 확장 프로그램은 분석, 추적, 광고를 위한 외부 서버 통신을 하지 않습니다.
- **사용자 설정만 저장합니다.** 사용자가 설정한 옵션(자동 열기, 배지 표시 등)은 `chrome.storage.sync`에 저장되어 브라우저 계정에 동기화되며, 설정 누락이나 일시적 동기화 실패에 대비한 마지막 정상 설정 사본은 `chrome.storage.local`에 저장됩니다. 설치·업데이트 진단에는 확장 버전과 이벤트 종류만 로컬에 기록하며 문서 URL이나 개인정보는 포함하지 않습니다.

### 권한 사용 목적

| 권한 | 사용 목적 |
|------|----------|
| `activeTab` | 현재 탭의 HWP 링크를 감지하기 위해 |
| `downloads` | HWP 파일 다운로드를 감지하여 뷰어에서 열기 위해 |
| `contextMenus` | HWP 링크 우클릭 메뉴를 제공하기 위해 |
| `clipboardWrite` | 문서에서 텍스트를 복사하기 위해 |
| `storage` | 사용자 설정을 저장하기 위해 |
| `host_permissions` | 웹페이지의 HWP 파일을 CORS 제한 없이 가져오기 위해 |

### 오픈소스

이 확장 프로그램은 MIT 라이선스로 공개된 오픈소스 소프트웨어입니다.
소스 코드: https://github.com/edwardkim/rhwp

---

## English

**rhwp - HWP Document Viewer & Editor** does not collect, store, or transmit any personal data.

### Data Processing Principles

- **Files are never sent to any server.** All HWP/HWPX file processing is performed locally in your browser using WebAssembly (WASM).
- **No personal data is collected.** We do not collect your name, email, location, browsing history, or any other personal information.
- **No external server communication.** This extension does not communicate with any external servers for analytics, tracking, or advertising.
- **Only user preferences are stored.** Preferences (auto-open, badge display, etc.) are saved in `chrome.storage.sync` for browser-account sync. A last-known-good settings snapshot is also stored in `chrome.storage.local` to recover from missing keys or temporary sync failures. Local install/update diagnostics contain only the extension version and event type, never document URLs or personal data.

### Permission Usage

| Permission | Purpose |
|-----------|---------|
| `activeTab` | To detect HWP links on the current tab |
| `downloads` | To detect HWP file downloads and open in viewer |
| `contextMenus` | To provide right-click menu for HWP links |
| `clipboardWrite` | To copy text from documents |
| `storage` | To save user preferences |
| `host_permissions` | To fetch HWP files from web pages without CORS restrictions |

### Open Source

This extension is open-source software licensed under the MIT License.
Source code: https://github.com/edwardkim/rhwp
