---
문서: 매뉴얼 현행화 감사 — 결정 요청 폼
작성일: 2026-04-23
기준: origin/devel 99a2e1f (PR #251 머지 + golden 재생성 시점)
목적: 매뉴얼 20개 중 구버전 정보를 담고 있는 파일을 식별하고 수정 범위를 결정
---

# 읽는 법

각 수정안 옆 `**결정**: [ ]` 에 다음 중 기입해 주세요.

| 코드 | 의미 |
|---|---|
| **수정** | 제시된 대안대로 변경 |
| **보류** | 현재 표현 유지 |
| **삭제** | 해당 문장/섹션 제거 |
| **자유** | 임의 지시 (예: "다르게: XXX") |

각 매뉴얼 맨 아래 **일괄 결정** 도 가능합니다.

---

# 최근 변화 요약 (매뉴얼 반영 대상)

| 변화 | 시점 | 반영 필요한 매뉴얼 |
|------|------|-------------------|
| **v0.7.3 / 확장 v0.2.1 배포** (Chrome/Edge 심사 통과) | 2026-04-19 ~ 21 | publish_guide, chrome_edge_extension_build_deploy |
| **Firefox 포팅 (rhwp-firefox)** | PR #169 (2026-04-19) | browser_extension_dev_guide, chrome_edge_extension_build_deploy |
| **rhwp-shared 도입 (공통 모듈)** | PR #214 | browser_extension_dev_guide |
| **SVG snapshot 하네스** | PR #181 | e2e_verification_guide, hyper_waterfall |
| **OLE/Chart/EMF 네이티브 렌더링** | PR #221 | (주로 코드 측, 매뉴얼 언급 확장 선택) |
| **테스트 수 증가**: 783 → 941 | | hyper_waterfall, publish_guide |
| **기여자 수 증가**: 1명 → 9명 이상 | | hyper_waterfall, onboarding_guide |

---

# [A] `publish_guide.md` — 버전 예시 구버전

## [A.1] line 76~79 · 버전 예시 테이블

```
| `Cargo.toml` | rhwp (Rust) + @rhwp/core 원본 | `version = "0.7.0"` |
| `rhwp-vscode/package.json` | VSCode 익스텐션 | `"version": "0.7.0"` |
| `npm/editor/package.json` | @rhwp/editor | `"version": "0.7.0"` |
| `rhwp-studio/package.json` | rhwp-studio (GitHub Pages 데모) | `"version": "0.7.0"` |
```

**권고**: 현행 예시 `0.7.3` 으로 갱신 (실제 현재 버전). 단순 예시 업데이트.

**결정**: [ 수정 ]

## [A.2] line 96~99 · 버전 업그레이드 예시

```
Cargo.toml:                  0.7.0 → 0.8.0
rhwp-vscode/package.json:    0.7.0 → 0.8.0
npm/editor/package.json:     0.7.0 → 0.8.0
rhwp-studio/package.json:    0.7.0 → 0.8.0
```

**권고**: `0.7.3 → 0.7.4` 또는 `0.7.3 → 0.8.0` 으로 갱신.

**결정**: [수정 ]

## [A.3] line 133~ · Cargo.toml 에디트 예시

```
version = "0.8.0"
```

현재 Cargo.toml 은 `0.7.3`. 예시는 "다음 릴리즈" 가정이므로 `0.7.4` 또는 `0.8.0` 유지 가능.

**권고**: 유지 또는 `0.7.4` 로 완화.

**결정**: [ 유지 ]

## [A.4] line 172, 186~188 · 커맨드 예시

```
git commit -m "v0.7.0 릴리즈 준비"
git tag v0.7.0
git push origin v0.7.0
gh release create v0.7.0 --title "v0.7.0 — 제목" --notes "릴리즈 노트"
```

**권고**: `v0.7.3` 으로 갱신 (현재 배포된 버전).

**결정**: [ 수정 ]

## [A.5] 브라우저 확장 버전 정책 섹션 누락

publish_guide 는 rhwp 라이브러리 / VSCode / npm 패키지만 다루고, **브라우저 확장 4종 (rhwp-chrome/firefox/safari) 의 버전 정책** 은 없습니다. v0.2.1 사이클에서 **라이브러리 v0.7.3 ≠ 확장 v0.2.1 이원화** 가 확정된 정책이므로 추가 필요.

**권고**: 신규 섹션 추가 — "브라우저 확장 버전 정책 (v0.2.x · 라이브러리와 이원화)".

**결정**: [ 추가 ]

**일괄 결정 (A 전체)**: [ ]

---

# [B] `chrome_edge_extension_build_deploy.md` — 이름·내용 Firefox 미반영

## [B.1] 문서 제목·범위

현재 제목: "Chrome/Edge 확장 프로그램 빌드 및 배포 매뉴얼"
실제 상태: **Firefox (rhwp-firefox) + Safari (rhwp-safari) 까지 존재**. 하지만 본 매뉴얼은 Chrome/Edge 전용.

**권고 옵션**:
- **옵션 1**: 제목을 "브라우저 확장 빌드 및 배포 매뉴얼" 로 확장하고 Firefox/Safari 섹션 추가
- **옵션 2**: Firefox/Safari 는 별도 매뉴얼 신설 (`firefox_extension_build_deploy.md`, `safari_extension_build_deploy.md`)
- **옵션 3**: Chrome/Edge 매뉴얼 유지 + 영역별 매뉴얼 3개 병존 (Firefox 는 신규 작성)

제 권고는 **옵션 1**: 3개 확장이 비슷한 빌드 파이프라인을 공유하므로 통합 매뉴얼이 유지 비용 낮음. 브라우저별 섹션 분리.

**결정**: [옵션1 ]

## [B.2] 빌드 크기 (2.4절)

```
| WASM 바이너리 | ~3.3MB |
| 폰트 | ~9MB |
| JS/CSS/HTML | ~4MB |
| 전체 | ~17MB |
```

현재 WASM 은 `3.9MB` (PR #221 EMF/OLE/Chart 포함). 그에 따라 전체도 달라짐.

**권고**: 실측치로 갱신 (`cd rhwp-chrome/dist && du -sh` 결과 반영).

**결정**: [ 수정 ]

## [B.3] 테스트 페이지 (3.4절)

```
| 테스트 | 검증 항목 |
| 01-auto-detect.html | ... |
| 02-data-hwp-protocol.html | ... |
| 03-dynamic-content.html | ... |
| 04-devtools.html | ... |
| 05-gov-site-sim.html | ... |
```

현재 5개 나열. rhwp-chrome/test/ 에는 **06-security.html** 이 있고, rhwp-firefox/test/ 는 같은 6개 + `index.html` 허브. 6번 누락.

**권고**: `06-security.html` 추가 (보안 검증 페이지).

**결정**: [ 추가 ]

## [B.4] CSP 예시 (문제 해결 섹션)

```
"content_security_policy": {
  "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
}
```

현재 manifest.json 은 더 엄격한 CSP 적용 가능. 실제 manifest 와 대조 필요.

**권고**: 현재 `rhwp-chrome/manifest.json` 의 실제 CSP 값으로 갱신.

**결정**: [ 수정 ]

**일괄 결정 (B 전체)**: [ ]

---

# [C] `browser_extension_dev_guide.md` — Firefox · rhwp-shared 미반영

작성일 2026-04-09, 대상: "Safari/Chrome/Edge". Firefox 미포함.

## [C.1] 문서 제목·대상

현재: "브라우저 확장 프로그램 개발 가이드 (Safari/Chrome/Edge)"

**권고**: "(Safari/Chrome/Edge/Firefox)" 로 확장.

**결정**: [ 수정 ]

## [C.2] Service Worker vs Background Scripts 표

현재 표는 Chrome/Edge · Safari 2열. Firefox 열 누락.

**권고**: Firefox 열 추가. Firefox MV3 는 Event Page 방식 (`background.scripts` + `type: module`).

**결정**: [ 수정 ]

## [C.3] `chrome.*` ↔ `browser.*` 네임스페이스 섹션 누락

PR #169 (Firefox 포팅) 의 핵심 교훈 — Chrome 은 `chrome.*`, Firefox 는 `browser.*` (Promise 지원). 매뉴얼에 반드시 들어가야 할 주제.

**권고**: 신규 섹션 추가 — "브라우저 API 네임스페이스 차이 (`chrome.*` vs `browser.*`)".

**결정**: [ 수정 ]

## [C.4] `rhwp-shared/` 공통 모듈 섹션 누락

PR #214 로 도입된 `rhwp-shared/sw/download-interceptor-common.js` (심볼릭 링크 + `dereference: true` 패턴).

**권고**: 신규 섹션 — "3개 확장 간 공통 모듈 공유 (rhwp-shared + symlink)".

**결정**: [ 수정 ]

## [C.5] Firefox 의 `onCreated` + `onChanged` 2단계 감지 패턴

Chrome 의 `onDeterminingFilename` 과 다름. PR #214 에서 확립된 패턴.

**권고**: 신규 섹션 — "다운로드 가로채기의 Chrome/Firefox 구조 차이".

**결정**: [ 수정 ]

**일괄 결정 (C 전체)**: [ ]

---

# [D] `hyper_waterfall.md` — 수치 구버전

## [D.1] line 246 · 테스트 수치

```
- **783+ 테스트**, Clippy 경고 0건
```

현재 **941 테스트**.

**권고**: `941+ 테스트` 로 갱신.

**결정**: [ 수정 ]

## [D.2] line 248 · "1인 개발" 문구

```
- **1인 개발** (+ Claude Code AI)
```

v0.2.1 사이클에서 외부 기여자 9명 합류. 더는 "1인 개발" 만으로는 정확하지 않음.

**권고 옵션**:
- **옵션 1**: "1인 메인테이너 + Claude Code AI + 외부 기여자 9명 (v0.2.1 기준)"
- **옵션 2**: "메인테이너 + Claude Code AI + 커뮤니티 기여자" (수치 없이)
- **옵션 3**: 그대로 유지 (핵심 아키텍처는 1인 메인테이너)

**결정**: [  옵션1 ]

## [D.3] line 264 · "v0.6.0 릴리즈" 언급

로드맵/이정표 맥락. v0.5.0 → v1.0.0 구도가 이미 README 에 확립됨 (`뼈대 → 조판 → 협업 → 완성`). v0.6.0 은 중간 단계로 실제 배포되지 않았음.

**권고**: "v0.5.0 → v0.7.x → v1.0.0" 같이 현재 구도로 갱신 또는 해당 줄 삭제.

**결정**: [ 수정 ]

**일괄 결정 (D 전체)**: [ ]

---

# [E] `e2e_verification_guide.md` — SVG snapshot 하네스 미반영

## [E.1] SVG snapshot 하네스 섹션 부재

PR #181 로 도입된 `tests/svg_snapshot.rs` + `tests/golden_svg/` 이 매뉴얼에 없음. 최근 PR #221 / #251 머지 후 **두 번 연속 CI 가 golden 재생성 누락으로 실패**한 교훈을 기록해야 함.

**권고**: 신규 섹션 추가 — "SVG 회귀 검증 (Rust 유닛 테스트 기반)"
- 하네스 위치: `tests/svg_snapshot.rs`
- Golden: `tests/golden_svg/*/page-N.svg`
- 재생성: `UPDATE_GOLDEN=1 cargo test --test svg_snapshot`
- **경고**: 렌더러 / 레이아웃 영향 PR 머지 후에는 반드시 `cargo test --test svg_snapshot` 실행, 실패 시 `UPDATE_GOLDEN=1` 재생성 + 결정성 재확인 + 커밋 (PR #221, #251 머지 후 2회 재생성 이력 있음)

**결정**: [ 신규 섹션 추가 ]

## [E.2] 한컴 PDF Visual Diff 계획 언급

이슈 #253 으로 Visual Diff 하네스 구상 진행 중. 매뉴얼에 "향후 한컴 PDF 기준 Visual Diff 하네스 도입 예정" 언급 가치 있음.

**권고**: "향후 작업" 섹션에 이슈 #253 링크 추가.

**결정**: [ 수정  ]

**일괄 결정 (E 전체)**: [ ]

---

# [F] `onboarding_guide.md` — 미세 갱신

`onboarding_guide.md` 에서 발견된 outdated 항목은 많지 않음:

## [F.1] line 291~292 · E2E 모드 설명

```
| `--mode=headless` | CI 자동화 | WSL2 내부 Chrome |
| `--mode=host` | 시각 확인 | 호스트 Windows Chrome CDP |
```

작업지시자 환경(Windows + WSL2) 기준 서술. 맥/리눅스 신규 기여자에게는 덜 명확.

**권고**: "호스트 Windows Chrome" → "호스트 브라우저 (Windows Chrome / macOS Chrome / Linux Chromium)".

**결정**: [ 수정 ]

## [F.2] `MEMORY.md` 또는 기여자 감사 섹션

v0.2.1 외부 기여자 9명 합류. onboarding 문서에 기여자 환영 메시지 업데이트 여부.

**권고**: 보류 가능. 별도 CONTRIBUTING.md 와 중복 가능성.

**결정**: [ ]

**일괄 결정 (F 전체)**: [ 수정 ]

---

# [G] 신규 매뉴얼 제안

## [G.1] `firefox_amo_submission_guide.md` (신규)

오늘 Firefox AMO 등록을 준비하신다고 하셨습니다. 작업지시자가 AMO 등록 과정에서 겪는 단계를 매뉴얼로 캡처하면 다음 확장(예: Safari App Store) 에도 재활용 가능. Chrome/Edge 심사 통과 후 이미 `chrome_edge_extension_build_deploy.md` 가 있으므로 대응 매뉴얼 신설이 자연스러움.

**권고**: Firefox AMO 등록 후 작성 (실제 경험 기반).

**결정**: [ 보류 ]

## [G.2] `pr_review_workflow.md` (신규)

PR 처리 절차 (리뷰 → 머지 → golden 재생성 → 이슈 close → archives 이동) 가 최근 수 건 PR 에서 반복됨. 매뉴얼로 정리 가치 있음.

**권고**: 별도 사이클에서 고려.

**결정**: [ 수정 ]

**일괄 결정 (G 전체)**: [ ]

---

# 결정 후 절차

작업지시자 결정 수신 후:

1. 일괄 수정 (각 매뉴얼에 대해)
2. main 직접 커밋 + push
3. devel cherry-pick + push
4. 본 문서 자체도 후속 기록용으로 `mydocs/feedback/archives/` 또는 `mydocs/manual/memory/` 로 이동

결정 권고 요약:
- **A** (publish_guide 버전 예시): **수정** (A.1, A.2, A.4) + **추가** (A.5)
- **B** (chrome_edge_deploy): **옵션 1** (통합 매뉴얼로 확장)
- **C** (browser_extension_dev): **수정** (C.1~5 모두)
- **D** (hyper_waterfall): **수정** (D.1 테스트 수치) + **수정 옵션 2** (D.2 기여자) + **삭제** (D.3 v0.6.0)
- **E** (e2e_verification): **추가** (E.1 + E.2)
- **F** (onboarding): **수정** (F.1) · F.2 보류
- **G** (신규): G.1 은 AMO 완료 후, G.2 는 보류
