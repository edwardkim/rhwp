# Task M100 #2125 Stage 1 완료 보고 — font ownership 기준선

- 이슈: #2125
- 브랜치: `task2125-assets-fonts-canonical`
- source 기준: `upstream/devel` `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- 계획 commit: `ecb3f70b9ae2419ff2994d2370d5e9c0dbd26207`
- 완료일: 2026-07-13
- 상태: Stage 1 완료, Stage 2 승인 대기

## 1. 수행 내용

1. `web/fonts`의 36개 WOFF2 filename, bytes, SHA-256을 전수 수집했다.
2. #2124 공식 snapshot과 현재 source를 비교했다.
3. #2190 / PR #2196 이후 승인된 Noto Sans KR delta를 분리했다.
4. source path, runtime URL, history-only reference를 분류했다.
5. Studio, Chrome, Firefox, Safari, VS Code, npm/editor, legacy `/web` ownership을 고정했다.
6. CI workflow trigger와 frontend detector의 `assets/fonts` 누락을 확인했다.

## 2. 기준선 결과

| 항목 | 결과 |
|------|------|
| WOFF2 | 36개 |
| total bytes | 22,651,296 |
| base source diff | `e750e02f` 대비 0 |
| #2124 대비 mismatch | `NotoSansKR-Regular.woff2` 1개 |
| mismatch provenance | #2190 / PR #2196 KS 기호 범위 보강 |
| 나머지 WOFF2 | 35개 bytes/hash 동일 |
| canonical 목표 | `assets/fonts` |
| Studio target | `rhwp-studio/public/fonts -> ../../assets/fonts` |
| legacy target | `web/fonts -> ../assets/fonts` compatibility link |

전체 manifest와 dependency graph는
`mydocs/tech/task_m100_2125_font_ownership.md`에 기록했다.

## 3. 추가 발견

`ci.yml`의 preflight detector만 고쳐서는 충분하지 않다. workflow push/PR trigger가 `assets/**` 전체를
`paths-ignore`하므로 `assets/fonts`만 바뀐 PR에서는 workflow 자체가 시작되지 않는다.

Stage 2에서는 다음을 원자적으로 반영한다.

- broad `assets/**` ignore를 현재 비런타임 asset 하위 경로로 축소
- detector에 `assets/fonts/` 추가
- Render Diff에 `assets/fonts/**` 반영
- build/test/tool source path와 binary move를 함께 변경

## 4. 검증

| 검증 | 결과 |
|------|------|
| `git diff e750e02f --` source 대상 파일 | 차이 0 |
| current count/bytes/hash 수집 | PASS |
| #2124 fontAssets 대조 | 설명되는 mismatch 1, 미설명 mismatch 0 |
| non-doc `web/fonts` reference 전수 조회 | 완료 |
| runtime `fonts/...` consumer 분류 | 완료 |
| `git diff --check` | Stage 문서 작성 후 최종 확인 예정 |

Stage 1은 조사·문서 단계이므로 build, E2E, font 이동은 실행하지 않았다.

## 5. Stage 2 진입 판단

ownership, manifest, 소비자 경로와 중단 조건이 모두 고정됐다. 기술적 blocker는 없다.

작업지시자 승인 전에는 다음을 수행하지 않는다.

- `web/fonts` -> `assets/fonts` 이동
- 두 symlink 변경
- Chrome/Firefox/VS Code build source 변경
- test/metrics/subset tool 변경
- CI/Render Diff workflow 변경
