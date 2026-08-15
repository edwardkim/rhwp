---
kind: pr-review
status: approved
pr: 4857
issue: 4823
author: jangster77
base: devel
head: codex/4823-conditional-document-webfonts
last_verified: 2026-08-15
---

# PR #4857 자체검토 - 문서별 조건부 웹폰트 로드

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#4857](https://github.com/edwardkim/rhwp/pull/4857) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `codex/4823-conditional-document-webfonts` |
| head SHA | `346035fcecb7a96d4e08446047053d848e739cd5` |
| 규모 | 대형 PR: 웹폰트 조사 산출물, Studio 로더, E2E 계약을 함께 포함 |
| mergeable | `MERGEABLE` |
| merge state | `CLEAN` (최신 head 필수 CI 통과) |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 위 상태값은 문서 작성 시점의 참고값이며,
merge 직전에 최신 head, GitHub Actions, mergeability를 다시 확인해야 한다.

## 관련 이슈와 범위

[Issue #4823](https://github.com/edwardkim/rhwp/issues/4823)의 목표는 문서가 선언한 글꼴이 시스템에
없을 때에만 검증된 웹폰트 CDN을 선택하도록 만드는 것이다.

- `font-loader.ts`는 시스템 감지 글꼴을 건너뛰고 문서 요청 글꼴만 `FontFace`로 등록한다.
- 조사 TSV에서 라이선스·사용 가능으로 판정된 80개 웹폰트 매핑을 생성 스크립트로 반영한다.
- KoPub, KoPubWorld, 정부상징서체 및 검증된 Noonnu 별칭을 포함한다.
- local font 탐색은 웹폰트 카탈로그 등록 자체를 시스템 설치 증거로 오인하지 않는다.
- 브라우저 콘솔의 `[FontLoader][debug]` 로그가 시스템 글꼴, CDN 후보, URL별 시작·성공·실패를 남긴다.
- 조사 스크립트, 보고서, TSV와 실행 로그는 재현 증적으로 포함한다.

Rust·renderer·WASM 소스, 기준 PDF, fixture는 변경하지 않았다.

## 규모와 검토 경계

1,000줄을 초과하는 대형 PR이지만 대다수는 조사 TSV·실행 로그의 갱신이다. 코드 검토 범위는 다음으로
한정했다.

- `rhwp-studio/src/core/font-loader.ts`
- `rhwp-studio/src/core/local-fonts.ts`
- `rhwp-studio`의 조건부 웹폰트·local font·font substitution 테스트
- `scripts/generate_document_webfont_catalog.mjs`
- `scripts/survey_korea_downloads_font_jsdelivr.mjs`

CDN 매핑은 HTTPS URL과 실제 응답을 조사 증적에서 확인한 항목만 등록한다. 라이선스 검토 상태의
공급 경로는 자동 등록하지 않는다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `npx tsc --noEmit && npm test` | 최종 Studio 검증: 935 passed, 1 skipped, 0 failed |
| `npm run build` | 통과 |
| `npm run e2e:renderer-contract` | 통과: 세로 글자 `inkMaskMaxDiffRatio`는 `0.005` 또는 `0.006`만 허용 |
| `npm run e2e:manifest-check` | 통과: renderer baseline manifest 102/102 추적 행 확인 |
| `table-border-style` baseline | 통과: CanvasKit compat/default 최대 비율 `0.005342831700801425`, `0.006` 예산 이내 |
| 브라우저 기동 | `http://127.0.0.1:7700/`에서 Studio UI와 WASM 초기화 확인 |
| 콘솔 진단 | CDN 후보, URL별 로드 시작·성공, `2개 성공, 0개 실패` 확인 |
| Rust/WASM 재빌드 | Rust 파일 변경이 없어 미수행 |
| visual sweep/PDF | renderer 출력·시각 fixture 변경이 없어 미적용 |
| GitHub Actions (최신 head) | 통과: Canvas visual diff, CodeQL 3축, Native Skia, test archive a/b/slow, 일반·slow test shard와 Build & Test 성공 |
| 정책상 skip | 정상: `Frontend unit gates`, `WASM Build`는 변경 영향 정책에 따른 `SKIPPED`이며 실패가 아님 |

로컬 `npm ci` 뒤 macOS ARM용 선택 의존성 `lightningcss-darwin-arm64`가 누락되어 첫 production
build가 실패했다. lockfile은 변경하지 않고 로컬 의존성을 복구한 뒤 같은 `npm run build`가 통과했다.

## 위험과 후속 조건

- 시스템 글꼴의 존재 여부와 CDN 응답은 사용자 환경·네트워크에 따라 달라질 수 있다. 외부 웹폰트는
  기존 `disableExternalWebFonts` 옵션으로 비활성화할 수 있다.
- `table-border-style`의 글꼴 raster 예산은 실측된 alias 적용 값만 반영한다. `0.005`와 `0.006` 외의 값은
  계약 검증에서 거부하고, 글꼴 미적용 값 `0.0131384`는 허용하지 않는다.
- 최신 head의 전체 GitHub Actions가 통과했고 merge 상태가 `CLEAN`이다. collaborator self PR이며
  리뷰어 지정 없이 작업지시자 승인으로 admin merge를 진행한다.

## 최종 권고

**병합 권고.** 최신 head `346035fcecb7a96d4e08446047053d848e739cd5`는 `CLEAN`/`MERGEABLE`이고
전체 GitHub Actions가 통과했다. 작업지시자의 승인에 따라 `--admin` 병합을 진행한다.
