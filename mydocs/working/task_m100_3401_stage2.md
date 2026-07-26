# v0.8.1 릴리즈 2단계 — 버전 갱신과 릴리즈 문서

Issue: #3401
브랜치: `task/3401-release-v0.8.1`
커밋: `973bff576`

## 1. 버전 갱신 — 9파일

계획서에는 10파일로 적었으나 실제 대상은 9파일이다. rhwp-chrome 과 rhwp-edge 는 한
코드베이스를 공유하므로 별도 manifest 가 없다(배포 시 zip 을 복사해 쓴다).

| 구분 | 파일 | 결과 |
|---|---|---|
| 라이브러리 | `Cargo.toml` (`[package]` 3행) | 0.8.1 |
| | `rhwp-vscode/package.json` | 0.8.1 |
| | `npm/editor/package.json` | 0.8.1 |
| | `rhwp-studio/package.json` | 0.8.1 |
| 확장 manifest | `rhwp-chrome/manifest.json` | 0.8.1 |
| | `rhwp-firefox/manifest.json` | 0.8.1 |
| | `rhwp-safari/src/manifest.json` | 0.8.1 |
| 확장 package | `rhwp-chrome/package.json` | 0.8.1 |
| | `rhwp-firefox/package.json` | 0.8.1 |

`git diff` 로 9파일 9줄이 전부 version 필드임을 확인했다. `Cargo.lock` 의 `rhwp` 항목은
빌드로 자동 갱신됐다. `pkg/package.json` 은 직접 편집하지 않는다(`prepare-npm.sh` 생성).

## 2. CHANGELOG — 3종

| 파일 | 내용 |
|---|---|
| `CHANGELOG.md` | 0.8.1 항목 — HWP3 렌더 정정, CLI 계약 정합, CLI 신규 기능, studio, 의존성, 알려진 문제 |
| `CHANGELOG_EN.md` | 동일 구조 영문 |
| `rhwp-vscode/CHANGELOG.md` | 라이브러리 v0.8.1 반영 + TypeScript 7 빌드 보정 |

E2E `issue-2214` 실패(#3412)를 **"알려진 문제"** 절에 명시했다. 회귀 여부 미확정이라는
사실도 함께 적어 확정 서술을 피했다.

## 3. THIRD_PARTY_LICENSES 정정

`Cargo.lock` 실측과 대조해 다음을 정정했다.

| 항목 | 종전 | 정정 |
|---|---|---|
| base64 | 0.22.1 | **0.23.0** |
| snafu | 0.9.1 | **0.9.2** |
| 기준 파일 표기 | `rhwp` v0.7.17 | **v0.8.1** |

lock 에는 base64 0.22.1 도 남아 있으나 이는 resvg/usvg 의 전이 의존이다. 이 표는 직접
의존성 목록이므로 0.23.0 이 맞다.

## 4. 스토어 제출 문서 — 4종 신규

| 파일 | 용도 |
|---|---|
| `mydocs/feedback/chrome-0.8.1_kor.md` | Chrome Web Store 한국어 |
| `mydocs/feedback/chrome-0.8.1_eng.md` | Chrome Web Store 영어 |
| `mydocs/feedback/edge-0.8.1_reviewer_notes.md` | Edge 심사 노트 |
| `mydocs/feedback/firefox-0.8.1_amo_notes.md` | Firefox AMO 검토 노트 |

Edge·AMO 노트에는 가이드가 요구하는 항목을 유지했다 — `<all_urls>` 필요 사유, 링크 감지·
배지·우클릭 메뉴 범위, 새 외부 endpoint 없음, WASM 로컬 처리, **No new permissions**.
AMO 노트의 source zip 명령은 0.8.1 파일명으로 갱신했고 200MB 제한과 제외 규칙 설명을
유지했다.

사용자 대상 문구는 기술 용어를 피해 작성했다 — "글맵시(꾸민 제목 글씨)", "도형 내부의
빈 구멍이 채워져 보이던 문제" 등.

## 5. 갱신 대상이 아니라고 판단한 것

- **README 3종**(rhwp-vscode, npm, npm/editor): 가이드 3단계의 필수 5항목(기능·폰트
  가이드·Third-Party·상표·Notice)을 이미 모두 갖췄고, 하드코딩된 버전 문자열이 없다.
- **package-lock.json 의 자체 `version` 필드**: studio 0.7.19, chrome/firefox 0.2.8 등
  옛 값이 남아 있으나 v0.8.0 릴리즈도 같은 상태로 배포됐다. `npm ci` 무결성 검사 대상이
  아님을 rhwp-chrome 에서 실측 확인(exit 0)했고, 스토어 심사는 manifest 기준이다.
  이번 릴리즈 범위를 넘는 정리이므로 건드리지 않았다.

## 6. 검증

| 항목 | 결과 |
|---|---|
| `cargo build` | 통과 — `Compiling rhwp v0.8.1` |
| `Cargo.lock` rhwp 버전 | 0.8.1 자동 갱신 |
| `rhwp-chrome` `npm ci` | exit 0 |
| 변경 규모 | 18파일 315+ 13- |

## 7. 다음 단계

3단계 — devel 대상 PR 생성·merge. **PR 생성은 별도 승인이 필요하다**(내부 타스크 PR 승인
규약). PR 직전 전체 CI 성격의 검증도 별도 명시 승인 대상이다.
