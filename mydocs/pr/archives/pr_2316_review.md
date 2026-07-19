# PR #2316 검토 — legacy /web 개발 앱과 current tooling 결합 제거

- PR: https://github.com/edwardkim/rhwp/pull/2316 (postmelee, collaborator)
- 이슈: #2313 (assignee postmelee, OPEN) / 관련 #2022 #2125 #2124
- base=devel, head=postmelee:task2313-legacy-web-removal (3커밋)
- 상태: **CONFLICTING** (orders 문서 1건, add/add) / maintainerCanModify=true
- CI (head, same-base): CodeQL·Frontend package gates·Native Skia 전 pass

## 변경 본질

repository 가 소비하지 않는 tracked `web/` 18 entries (legacy HTML/JS 앱,
Python HTTPS 서버, cert/key, 생성물 glue) 제거 + CI detector·metrics·font
contract 의 `/web` 결합 정리 + 한/영 local server 매뉴얼 Studio 현행화.
Git history 를 archive 로 사용 (stub/복사본 없음). Rust/WASM·에디터
공개 계약·폰트 바이너리는 불변.

## 구조 검토

- `.github/workflows/ci.yml`: frontend prefix 목록에서 `web/` 1줄 제거 — 정합
- `scripts/frontend-metrics.mjs`: `legacy-web` include group + web 전용
  exclude 5건 제거 — 정합
- `scripts/frontend-font-assets.test.mjs`: 심링크 계약을 Studio canonical
  단독으로 축소 (`web/fonts` 링크 항목 제거) — 정합
- 잔존 참조 스캔: workflows/scripts/package.json 에 `web/` 참조 0건,
  `git ls-files web/` 0건. studio 소스의 "web-substitute" 등은 무관 문자열
- 컨트리뷰터 자체 하이퍼-워터폴 문서(plans/working/report, stage 1~5) 동봉

## 로컬 재실증 (merged tree = devel 6d57ed81 + head)

| 게이트 | 결과 |
|--------|------|
| rust 영향 | diff 0 (src/Cargo 불변 — devel 최신 rust 그대로) |
| font asset contract | **4/4** |
| frontend-metrics | 정상 실행 (214 files, legacy group 소멸) |
| studio | npm ci + 단위 **307/307** + production build PASS |
| @rhwp/editor | npm ci + **15/15** |

## 충돌 해소안

`mydocs/orders/20260717.md` add/add — 컨트리뷰터가 자기 작업 기록으로 만든
오늘할일과 메인테이너 파일의 충돌. **union 해소**: 메인테이너 본문 유지 +
컨트리뷰터 #2313 행을 "컨트리뷰터 진행 (PR 연동)" 절로 편입.
maintainerCanModify=true → 해소 커밋을 head 브랜치에 직접 push
(fork LFS locksverify false 필요 시 적용) 후 merge.

## 판단

**merge 권고.** 소비되지 않는 legacy 결합 제거로 frontend 표면이 단순해지고
(Total CC -828, 함수 -149 — 전량 legacy 그룹), 계약·게이트가 Studio 단독
기준으로 명료해진다. 위험 표면은 문서/스크립트/CI 한정이며 재실증 전부 green.


---

## 정밀 평가 (작업지시자 요청, 2026-07-17)

16,126 삭제 라인의 안전성을 4개 축으로 전수 실증했다.

### 1. 소비처 전수 스캔 — 소비 0 확인

| 소비 후보 | 실측 |
|-----------|------|
| GitHub Pages 배포 (`deploy-pages.yml`) | `rhwp-studio/dist` 만 업로드 — web/ 미배포 |
| npm 배포 (`npm-publish.yml`) | `wasm-pack build` 로 pkg/ 신선 생성 — tracked `web/rhwp.js` 미소비 |
| release-binary / render-diff / full-sweep | web/ 참조 0 |
| 확장 4종 빌드 (chrome/firefox/vscode build·config) | web/ 참조 0 |
| studio 소스/e2e | import 0 — "web/editor.html 포팅" 등 유래 주석뿐 |
| tracked 참조 전체 | ci.yml·metrics·font-test 3곳 = 본 PR 이 제거하는 결합 그 자체 |

### 2. 제거물 성격 분류 (18 entries, 616KB)

- **stale 생성물 377KB** (`rhwp.js`/`rhwp.d.ts`/`rhwp_bg.wasm.d.ts`): wasm-pack
  산출물의 tracked 복사본 — 실배포는 항상 pkg/ 신선 빌드라 이중 진실이었음
- **legacy 앱 224KB** (editor/app/format_toolbar/char_shape_dialog 등 10파일):
  마지막 실질 변경 **2026-04-07** (#71 XSS 수정) — 3개월+ 휴면, Studio 가 기능 상위
- **인프라 잔재**: Python HTTPS 서버 + **tracked private key**(localhost-key.pem)
  — 자가서명 로컬용이나 저장소 위생상 제거가 개선
- **호환 심링크** `web/fonts`: canonical 은 `assets/fonts`, Studio 링크는 별도 존치

### 3. metrics 정밀 대조 (devel 6d57ed81 vs merged tree, 동일 도구)

| 총량 | devel | merged | delta | 대조 |
|------|------:|-------:|------:|------|
| Reported functions | 2520 | 2371 | **-149** | legacy 그룹 함수와 일치 |
| Total CC | 13115 | 12287 | **-828** | legacy 그룹 CC 828 과 정확 일치 |
| CC>25 count / sum | 73 / 4473 | 69 / 4266 | -4 / **-207** | legacy 그룹 4건/207 과 정확 일치 |
| CC>100 / Max | 7 / 453 | 7 / 453 | 0 | 불변 |

**non-legacy 7개 그룹(Studio/Chrome/Firefox/Safari/Shared/VSCode/npm)의
Files·Lines·Functions·CC 전 수치 완전 동일** — 제거가 legacy 그룹에 100%
국한됨을 함수 단위로 실증. (PR 주장 Top20 -38 vs 실측 -41 은 base 전진
(#2318 TS 추가 등)에 따른 순수 재계산 차이로 무해)

### 4. 문서 일관성

- 규범 매뉴얼(local_web_server ko/en) Studio Vite 로 갱신, `font_fallback_strategy`
  는 "당시 구현 경로 기록 보존" 주석 명시 — 역사 기록 원칙 부합
- 잔존 `web/` 언급 문서는 전부 기준 commit·작성일 박힌 스냅샷/아카이브
  (#1904/#2124/#2125 기준선 — PR 본문의 불변 선언과 일치)
- 복원성: stub 없이 git history 가 archive — `git checkout <sha> -- web/` 로 전량 복원 가능

### 평가 결론

제거 전량이 (a) 소비처 0, (b) 휴면 3개월+, (c) 함수 단위 metrics 로 legacy
그룹 100% 국한 입증, (d) 역사 기록·복원성 보존 — **merge 권고 유지**.
