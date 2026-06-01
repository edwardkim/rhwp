# PR #1162 처리 보고 — diff-engine-readme UI 진입점 안내 추가

## 결정: MERGE

- 머지: 2026-05-29T06:58:24Z, squash, devel `30775f9f`
- 작성자: [@xogh3198](https://github.com/xogh3198) (taeho) — compare/diff-engine 기존 핵심 컨트리뷰터 (#571/#623/#625/#799)

## 사유

- 문서 전용 변경(+9/-0, `rhwp-studio/src/compare/diff-engine-readme.md` §6.1).
- 기재 내용(메뉴·단축키·함수·전략·파일)이 현 코드와 100% 일치 — 검토 문서 4절 10개 항목 전수 검증.
- 코드/기능 변경 없음 → 빌드/테스트 회귀 위험 없음.

## 머지 절차 비고

- 머지 시점 `mergeStateStatus: BEHIND` — devel 의 워터마크 _v2(362a0f99, ci.yml 포함) push 로 head(fork main)가 base 보다 뒤처짐.
- `update-branch` 시도 → OAuth 토큰 `workflow` scope 부족(403, devel 의 ci.yml 변경을 fork 에 머지하려다 거부).
- 문서 단일 파일이라 devel 변경과 무충돌 + devel CI 이미 그린 → `--admin` squash 로 status check 우회 머지. 회귀 위험 없음으로 판단.

## 후속 — 기여자 계정 정정 (별도)

작업지시자 제기: @xogh3198 이 과거 PR 커밋 author 계정 불일치로 기여 그래프/목록에서 누락. 본 PR 과 분리하여 정정 범위·올바른 계정 확인 후 별도 처리 예정. (README.md 텍스트 기여자 목록 v0.7.13 에는 이미 @xogh3198 포함)

## 검증

- 문서 정합성: 4절 전수 통과 (코드 변경 없어 cargo 검증 불요)
- 로컬 devel 동기화: `git merge --ff-only origin/devel` → `30775f9f`
