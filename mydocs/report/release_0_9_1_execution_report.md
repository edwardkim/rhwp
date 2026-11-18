---
kind: snapshot
status: active
canonical: mydocs/manual/publish_guide.md
last_verified: 2026-11-19
---

# v0.9.1 배포 결과

v0.9.1의 공개 CLI, Studio, crates.io, npm, VS Code Marketplace와 Open VSX를 확인했다.
메인테이너는 Chrome·Edge·Firefox 스토어에 0.9.1을 수동 제출했다. Firefox의 0.9.1 공개 배포를 확인했다.
배포에 사용한 파일과 공개 다운로드의 해시, 패키지별 버전·소스는
[배포 증적](../pr/assets/release-0.9.1-execution/delivery-evidence.json)에 기록했다.

## 소스와 승격

- 기능 마감 기준 devel은 `197ce19e99`, 승격 기준 devel은 `08b38c12a75fa5cabfe5fbbfde9388aceb3fcca3`다.
  승격 workflow 12개와 정책 항목 15개를 모두 통과했다. 승격 검증 예외는 0건이다.
- main 승격 PR의 검사와 셀프 리뷰 이후 2026-11-18 23:32:33 KST에 병합했다.
  메인테이너는 이 PR에 한해 승인 리뷰 개수 요건에 예외를 승인했다.
  병합 커밋과 정식 태그는 `981ce66b4183b13c1a3b5907e04a8969c8c42034`다.
  [#9176 검토 기록](../pr/archives/pr_9176_review.md).
- 공개 릴리즈 노트는 기여자 19명과 Dependabot을 기록하고 PDF 가져오기 개발자 안내를 연결한다.
  [v0.9.1 릴리즈](https://github.com/edwardkim/rhwp/releases/tag/v0.9.1).

## 공개 채널 검증

| 채널 | 확인 결과 |
| --- | --- |
| GitHub Release | 5개 플랫폼의 CLI archive SHA-256을 확인했다. 공개 Linux x86_64 바이너리로 실제 794×1123 PNG를 내보냈다. [릴리즈](https://github.com/edwardkim/rhwp/releases/tag/v0.9.1) |
| GitHub Pages | 배포 ID `7886986139`가 태그와 같은 SHA를 가리킨다. 공개 HTML과 참조 JS·CSS의 HTTP 200을 확인했다. [배포 실행](https://github.com/edwardkim/rhwp/actions/runs/44325466203) |
| crates.io·docs.rs | 새 버전 8개를 태그 소스에서 게시했고 기존 버전 6개를 재사용했다. 14개 archive의 공개 인덱스·SHA-256과 docs.rs 지정 버전 빌드를 모두 확인했다. [공개 crate](https://crates.io/crates/rhwp/0.9.1) · [빌드](https://docs.rs/crate/rhwp/0.9.1/builds) |
| npm | `@rhwp/core`와 `@rhwp/editor` 0.9.1의 공개 설치·서명·provenance를 확인했다. Chromium에서 공개 패키지로 HWP 열기, HWP/HWPX 저장·재열기를 통과했다. [배포 실행](https://github.com/edwardkim/rhwp/actions/runs/44332298724) |
| VS Code Marketplace·Open VSX | 양쪽 공개 0.9.1 VSIX가 같은 CI 산출물과 바이트 단위로 일치했다. [Marketplace](https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode) · [Open VSX](https://open-vsx.org/extension/edwardkim/rhwp-vscode) |

[Release Binary 게시 증적](https://github.com/edwardkim/rhwp/actions/runs/44332298724/artifacts/14448456977)은
같은 태그 SHA에서 npm core·editor, VS Code Marketplace, Open VSX 네 채널의 게시 완료를 확인한다.
보존한 패키지별 소스는 공개 archive의 VCS 메타데이터를 기준으로 한다.

### 공개 Rust 패키지 소비

- Docker에서 공개 패키지만 사용해 Rust·Native CLI·PDF 구성에 걸친 36개 검사를 통과했다.
- Noto Sans KR 명시·자동 탐색으로 생성한 794×1122 PNG의 SHA-256이 같았다. 제목·본문·표의 한글을 직접 판독했다.
- 별도 공개 Rust WASM 소비 검사에서 `wasm-api`를 켜고 끈 두 구성을 각각 빌드하고 Chromium에서 HWP 열기·HWPX 저장·재열기를 확인했다.
- 재현 환경과 원시 결과는 저장소의 Git 제외 경로 `local/crates-io-consumption/`에 보존한다.
  재실행은 해당 경로의 `run.sh --version 0.9.1`을 사용한다.

## 브라우저 확장 스토어

정식 태그에서 스토어별 ZIP을 준비했다. 로컬 패키지 검사와 스토어 심사·공개 상태는 별도 시점에 기록한다.

| 대상 | 준비 및 검증 | 제출 안내 |
| --- | --- | --- |
| Chrome Web Store | 확장 로딩·다운로드 가로채기·수명주기 E2E 3개 통과. 메인테이너 로컬 모드 수동 테스트 통과. | [한국어](../feedback/chrome-0.9.1_kor.md) · [영어](../feedback/chrome-0.9.1_eng.md) |
| Microsoft Edge Add-ons | Chrome에서 검증한 ZIP과 바이트 단위로 같은 패키지. 메인테이너 로컬 모드 수동 테스트 통과. | [Edge 심사 노트](../feedback/edge-0.9.1_reviewer_notes.md) |
| Firefox AMO | 제출용 source ZIP에서 WASM과 확장을 재빌드했다. 실제 Firefox에서 문서 열기·편집·저장과 저장본 재열기를 확인했다. | [AMO 심사 노트](../feedback/firefox-0.9.1_amo_notes.md) |

Chrome·Firefox manifest는 v0.9.0 대비 버전만 달라졌다. ZIP별 SHA-256과 크기는
[배포 증적](../pr/assets/release-0.9.1-execution/delivery-evidence.json)의 `browser_store_preflight`에 기록했다.
파일은 로컬 `output/release-v0.9.1/browser-store/packages/`에 모았다.
제출은 [메인테이너 수동 절차](../manual/publish_guide.md#8단계-브라우저-확장-스토어-업로드)에 따른다.

### 2026-11-19 KST 제출 및 공개 확인

| 스토어 | 메인테이너 제출 | 공개 확인 |
| --- | --- | --- |
| Chrome Web Store | 0.9.1 제출 완료 | 공개 목록은 0.9.0으로 표시된다. [공개 목록](https://chromewebstore.google.com/detail/pgakpjflombjmehnebnbpnalhegaanag) |
| Microsoft Edge Add-ons | 0.9.1 제출 완료 | 공개 업데이트 응답은 0.9.0.0을 제공한다. [공개 목록](https://microsoftedge.microsoft.com/addons/detail/nfkdfobhmanddlhdbclkpoanbccpigcn) |
| Firefox AMO | 0.9.1 제출 완료 | 0.9.1이 공개됐고 XPI를 다운로드했다. [공개 목록](https://addons.mozilla.org/en-US/firefox/addon/rhwp-free-hwp-editor/) · [공개 XPI](https://addons.mozilla.org/firefox/downloads/file/5273191/rhwp_free_hwp_editor-0.9.1.xpi) |

Firefox 공개 XPI의 해시는 AMO 응답과 일치했다. 제출 ZIP과 비교해 107개 파일은 바이트가 같았고,
manifest는 서식만 바뀌었으며 JSON 값은 같다. Mozilla 서명 파일 5개가 추가됐다.
공개 XPI를 격리 Firefox에 설치해 HWP 열기·편집·저장 및 저장본 재열기를 통과했다.
메인테이너가 Firefox 0.9.1 업데이트가 표시되고 정상 적용되며, 업데이트 후 확장 기능이 동작하는 것을 직접 확인했다.
Chrome과 Edge는 0.9.1 공개 전환을 확인할 때까지 제출 완료 상태로 기록한다.

## 남은 검증

- [태그 CI](https://github.com/edwardkim/rhwp/actions/runs/44332297958)의 Native Skia는 최초 실행과
  승인된 한정 재실행 모두 45분 제한으로 취소됐다. 두 번째 실행에서 61건이 통과한 뒤 후속
  회귀 바이너리를 링크하던 중 종료됐다. A–E, WASM, Frontend와 lint는 성공했다.
  태그 CI 집계는 Native 취소로 실패했다. main 승격 PR의 CI와 공개 배포 검증은 성공했다.
  [재실행 로그](https://github.com/edwardkim/rhwp/actions/runs/44332297958/job/134937841802).
- 같은 SHA의 [태그 CI 3차 실행](https://github.com/edwardkim/rhwp/actions/runs/44332297958/attempts/3)도 Native Skia가 45분 제한에서 취소됐고 Build & Test 집계가 실패했다.
- [#9154](https://github.com/edwardkim/rhwp/issues/9154)는 main에서 정기 Fuzz가 실행된 결과를 확인할 때까지 열린 상태다.
