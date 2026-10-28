# v0.8.2 핫픽스 2단계 — 버전 갱신과 릴리즈 문서

Issue: #3445
브랜치: `task/3445-release-v0.8.2`
커밋: `28b9032a1`

## 1. 버전 갱신 — 9파일

계획대로 9파일을 0.8.1 → 0.8.2 로 올렸다. `git diff` 로 변경 라인 전수가 version 필드임을
확인했다(version 외 변경 0건).

| 구분 | 파일 |
|---|---|
| 라이브러리 | `Cargo.toml`(`[package]` 3행), `rhwp-vscode/package.json`, `npm/editor/package.json`, `rhwp-studio/package.json` |
| 확장 manifest | `rhwp-chrome`, `rhwp-firefox`, `rhwp-safari/src` |
| 확장 package | `rhwp-chrome`, `rhwp-firefox` |

`Cargo.lock` 의 `rhwp` 항목은 빌드로 자동 갱신됐다(`Compiling rhwp v0.8.2`).

## 2. CHANGELOG — 3종

| 파일 | 내용 |
|---|---|
| `CHANGELOG.md` | 0.8.2 — 확장 인쇄 복구, 렌더 정정, 알려진 문제 |
| `CHANGELOG_EN.md` | 동일 구조 영문 |
| `rhwp-vscode/CHANGELOG.md` | 라이브러리 반영 + **VS Code 확장은 인쇄 이슈 영향 없음** 명시 |

핫픽스의 성격을 앞세워 "브라우저 확장의 인쇄 기능이 동작하지 않던 문제를 복구한다.
v0.8.0 부터 영향받았다" 를 요약문에 두었다.

`rhwp-vscode/CHANGELOG.md` 에는 이번 본체가 브라우저 확장 한정임을 적었다. VS Code 확장
사용자가 자신과 무관한 수정을 릴리즈 노트에서 보고 혼란하지 않도록 한 것이다.

### 알려진 문제 2건 명시

- #3450 — studio E2E `print-pdf-issue3126` PDF 안내 모달 실패. 인쇄 surface 자체는 정상이며
  이번 범위 밖 변경에서 비롯했다. 근인 미진단.
- #3412 — studio E2E `issue-2214` 페이지 로컬 리페인트 계약 실패. v0.8.1 에서 이어지며 회귀
  여부 미확정.

두 건 모두 "확인하지 않은 것" 을 확정 서술로 바꾸지 않았다.

## 3. THIRD_PARTY_LICENSES

**기준 파일 표기만** v0.8.1 → v0.8.2 로 갱신했다. `git diff v0.8.1..HEAD -- Cargo.toml
Cargo.lock` 이 비어 있어 의존성 자체는 변경이 없다. v0.8.1 에서 base64·snafu 를 정정한 것과
달리 이번에는 라이브러리 목록을 손대지 않았다.

## 4. 스토어 제출 문서 — 4종 신규

| 파일 | 용도 |
|---|---|
| `mydocs/feedback/chrome-0.8.2_kor.md` | Chrome Web Store 한국어 |
| `mydocs/feedback/chrome-0.8.2_eng.md` | Chrome Web Store 영어 |
| `mydocs/feedback/edge-0.8.2_reviewer_notes.md` | Edge 심사 노트 |
| `mydocs/feedback/firefox-0.8.2_amo_notes.md` | Firefox AMO 검토 노트 |

인쇄 복구를 사용자 체감 문구로 앞세웠다 — "Ctrl+P로 인쇄할 때 '파일을 찾을 수 없음' 오류가
나던 문제를 정정했습니다."

Edge·AMO 노트에는 가이드 필수 항목을 유지했다 — `<all_urls>` 사유, 새 외부 endpoint 없음,
WASM 로컬 처리, **No new permissions**. AMO 노트의 source zip 명령은 0.8.2 파일명으로
갱신했고 200MB 제한·제외 규칙 설명을 유지했다. Edge 노트의 테스트 절차에는 이번 릴리즈의
초점이 4번 항목(Ctrl+P 인쇄)임을 덧붙였다.

## 5. 갱신 대상이 아니라고 판단한 것

- **README 3종**: v0.8.1 2단계에서 필수 5항목을 갖췄고 하드코딩된 버전 문자열이 없음을
  확인했다. 이번 릴리즈에서 README 관련 변경이 없어 재점검 대상이 아니다.
- **package-lock.json 자체 version 필드**: v0.8.1 과 동일한 판단. `npm ci` 무결성 검사
  대상이 아니며 스토어 심사는 manifest 기준이다.

## 6. 검증

| 항목 | 결과 |
|---|---|
| `cargo build` | 통과 — `Compiling rhwp v0.8.2` |
| `Cargo.lock` rhwp 버전 | 0.8.2 자동 갱신 |
| 변경 라인 검증 | version 외 변경 **0건** |
| 변경 규모 | 18파일 250+ 11- |

## 7. 다음 단계

3단계 — devel 대상 PR 생성·merge. **PR 생성은 별도 승인이 필요하다.** PR 직전 CI 성격의
검증도 별도 명시 승인 대상이다.
