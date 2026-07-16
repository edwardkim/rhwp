# PR #2254 검토 — assets/fonts canonical root 이전 (postmelee, refs #2125)

- 검토일: 2026-07-15 / base: devel / 69파일 +2,283/−63 / CI 12 green / BEHIND
- 구성: 순수 rename 37(R100) + R084 1(FONTS.md, 이동 중 갱신) + 수정 19
  (빌드/CI/문서) + 추가 12(호환 링크·계약 테스트·운영 문서).
- 요지: WOFF2 36개를 `assets/fonts` 단일 canonical 로 이전, 5개 소비자
  (Studio/legacy web/Chrome·Firefox/Safari/VS Code)의 source·copy 계약 정리.
  runtime URL·CSP·publicDir:false·@rhwp/editor 무의존 계약 전부 유지 선언.

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| **바이트 보존 독립 검증** — devel:web/fonts 36개 vs canonical SHA-256 | **불일치 0** (22,651,296 bytes, 개수·총량 PR 주장 일치) |
| 신설 계약 테스트 `frontend-font-assets.test.mjs` | 4/4 (inventory·링크·확장 dist 전체·VS Code 11개 subset) |
| `frontend-extension-dist.test.mjs` | 3/3 |
| Studio: npm ci + tsc + test + build | 클린 / 270/0 / 성공 — **dist/fonts 36개** (symlink 경유 복사 정상) |
| Chrome/Firefox `build.mjs` + VS Code compile | 전부 성공 |

> 처음 계약 테스트 1건 실패는 **검토 환경의 stale dist**(gitignored 산출물,
> #2196 보강 전 빌드) 원인 — 재빌드 후 전부 통과. PR 결함 아님. 신설
> 테스트가 stale dist 를 잡아낸다는 방증이기도 함.

## 구조 검토

- 호환 링크는 심볼릭 링크(`public/fonts → ../../assets/fonts` 등) — Linux/
  macOS/CI 정상. Windows 네이티브 git 은 developer mode 필요하나 본 저장소
  개발 흐름(WSL2/macOS)과 CI(Linux)에서 문제 없음. 운영 문서에 기록됨.
- CI 변경 감지: assets/fonts 가 frontend gate·Render Diff 를 트리거하도록
  확장 + "Verify font asset contracts" 스텝 신설 — 계약의 CI 상시화.
- extension 보안 계약(publicDir:false, CSP, WAR) 비접촉 — 빌드 스크립트의
  source 경로만 canonical 로 변경.
- Safari signed build 는 인증서 부재로 미실행(컨트리뷰터 명시) — unsigned
  Xcode build + .appex 해시 parity 로 대체, release gate 유지. 타당.

## 판단

**approve → merge 수용 권고.** BEHIND — head 가 postmelee fork 이므로
merged tree 선검증 후 admin merge (#2257 방식) 또는 collaborator 본인
rebase 요청. 이전 계열(#2125) 이슈는 close 키워드 없음 — refs 유지.
