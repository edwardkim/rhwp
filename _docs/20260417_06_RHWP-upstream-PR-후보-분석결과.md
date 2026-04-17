---
title: RHWP upstream PR 후보 분석결과
created: 2026-04-17 20:27

session_id: codex:019d9a20-7c77-7513-978d-506539b5103d
session_path: C:/Users/ahnbu/.codex/sessions/2026/04/17/rollout-2026-04-17T15-28-33-019d9a20-7c77-7513-978d-506539b5103d.jsonl


ai: codex
---

# RHWP upstream PR 후보 분석결과

## 발단: 사용자 요청

사용자는 현재 `local/task179-devel` 브랜치의 변경사항 중 업스트림(`upstream/main`)에 PR하면 좋을 것들을 검토해달라고 요청했다.

이번 문서는 그 검토 결과만 재사용 가능하게 남기기 위한 기록이다. 로컬 문서 운영 규칙, backlog, `doc-save-cp` 논의는 범위에서 제외했다.

## 작업 상세내역

현재 브랜치는 `upstream/main` 대비 6개 커밋 앞서 있었다.

- `9c8431e fix(studio): 열린 문서를 Ctrl+S로 바로 저장`
- `ea0fef0 feat(windows): HWP 더블클릭과 원본 저장 연동`
- `6aac49f docs(project): 문서 저장 위치 규칙 정리`
- `5f22179 fix(windows): launch manifest BOM으로 저장 실패 수정`
- `c3e9ba9 docs(report): 저장 실패 오류보고서 갱신`
- `5938480 docs(backlog): 루트 백로그 추가`

검토는 `git diff upstream/main..HEAD` 기준으로 파일 범위를 나누고, 각 커밋의 목적과 업스트림 적합성을 다시 비교하는 방식으로 진행했다.

- 제품 기능 후보: `rhwp-studio` 저장 개선, Windows 더블클릭/저장 bridge
- 로컬 전용 후보: `AGENTS.md`, `CLAUDE.md`, `BACKLOG.md`, `_docs/`
- 설치/배포 리스크 후보: `tools/` 아래 Windows 런처와 설치 스크립트

특히 Windows 경로는 단순 기능 추가가 아니라 설치 방식, 런처 의존 파일, preview 서버 교체까지 묶여 있어 PR 범위를 더 작게 쪼갤 필요가 있었다.

## 의사결정 기록

| 후보 | 사용자 가치 | PR 준비도 | 핵심 blocker | 판단 |
|------|-------------|-----------|--------------|------|
| `9c8431e` 열린 문서 `Ctrl+S` 저장 개선 | ✅ 높음 | ✅ 바로 가능 | 없음 | 업스트림 1차 PR 후보 |
| `ea0fef0` + `5f22179` Windows 더블클릭 + save bridge | ✅ 높음 | ⚠️ 정리 후 가능 | 런처 의존 파일 누락, installer 부작용, 범위 과대 | 후속 PR 후보 |
| `6aac49f`, `c3e9ba9`, `5938480` 문서/로컬 운영 변경 | ❌ 낮음 | ❌ 부적합 | 업스트림 제품 가치 없음 | PR 제외 |

<span style="color:#888">*정렬 기준: 업스트림 사용자 가치와 리뷰 통과 가능성을 함께 봤다. 기능 가치가 높아도 설치 부작용이나 로컬 규칙이 섞이면 우선순위를 낮췄다.*</span>

- 결정: 업스트림에는 먼저 `9c8431e` 범위만 분리해 PR하는 것이 가장 적절하다.
- 근거: `showOpenFilePicker()`로 연 문서를 같은 handle에 바로 저장하는 흐름은 일반 사용자 가치가 분명하고, Windows 전용 설치 도구 없이도 독립적으로 설명 가능하다.
- 근거: Windows double-click/save bridge는 기능 자체는 유효하지만, 현재 상태로는 `RHWPLauncher.exe`가 실행하는 `rhwp_launcher.ps1`이 tracked되지 않았고 설치 스크립트가 기존 파일 연결 정보를 삭제한다.
- 트레이드오프: 1차 PR을 작게 가져가면 리뷰와 병합 가능성은 높아지지만, 사용자가 체감하는 더블클릭 실행 흐름은 별도 2차 PR로 분리해야 한다.

## 검증계획과 실행결과

> compare-table 스킬 이모지 포맷 적용 (✅❌⚠️⏳)

| 검증 항목 | 검증 방법 | 결과 | 비고 |
|-----------|-----------|------|------|
| 업스트림 기준 커밋 범위 확인 | `git log --reverse --oneline upstream/main..HEAD` | ✅ 완료 | 6개 선행 커밋 확인 |
| 파일 범위 분류 | `git diff --name-status upstream/main..HEAD` | ✅ 완료 | 제품 기능, Windows 도구, 로컬 문서로 분류 |
| 저장 개선 PR 단독성 확인 | `git show --stat --summary 9c8431e` | ✅ 완료 | `rhwp-studio` 내부 저장 경로 개선으로 설명 가능 |
| Windows PR blocker 확인 | `git show --stat --summary ea0fef0`, `git show --stat --summary 5f22179` | ✅ 완료 | 런처/installer/preview 서버 변경이 함께 묶여 있음 |
| 런처 의존 파일 누락 확인 | `git ls-files tools/...`, `git status --ignored --short tools` | ✅ 완료 | `rhwp_launcher.ps1`는 ignored, tracked 아님 |
| 설치 스크립트 부작용 확인 | `tools/install_rhwp_openwith.ps1` 검토 | ✅ 완료 | 기존 `.hwp/.hwpx` HKCU 연결 삭제 로직 존재 |
| 현재 브랜치 검증 상태 확인 | `node --test ...`, `go test ./...`, `npm run build` | ✅ 완료 | 테스트 14개 통과, Go 테스트 통과, build 성공 |

## 리스크 및 미해결 이슈

- Windows PR은 현재 그대로 올리면 upstream에서 바로 재현 가능한 설치 경로가 되지 않는다. `RHWPLauncher.exe`가 요구하는 `rhwp_launcher.ps1`가 PR 범위에 없다.
- 설치 스크립트는 기존 사용자 연결 정보를 지우는 방식이라 upstream 배포 스크립트로는 위험하다.
- `file-save-flow.js` + `.d.ts` 조합은 strict TS 프로젝트 관점에서 임시 구조에 가깝다. 업스트림 PR 전에는 `.ts`로 합치는 편이 안전하다.
- `npm run preview`를 커스텀 preview server로 바꾼 결정은 기능상 필요가 있었지만, 업스트림 리뷰에서는 범위가 넓다고 볼 가능성이 높다.

## 다음 액션

- 1차 PR은 `9c8431e` 저장 개선 범위만 다시 정리해 별도 브랜치로 분리한다.
- 2차 PR을 준비하려면 Windows 경로에서 `rhwp_launcher.ps1`를 tracked 파일로 포함하고 installer를 비파괴적으로 바꾼다.
- Windows 경로는 가능하면 `preview` 스크립트 변경과 설치 도구 변경을 다시 나눠 리뷰 단위를 줄인다.
- 로컬 문서와 backlog, 프로젝트 운영 규칙 파일은 업스트림 PR에서 계속 제외한다.

## 참고 자료

| 출처 | 용도 |
|------|------|
| `upstream/main..HEAD` git log/diff | 업스트림 대비 선행 커밋과 파일 범위 확인 |
| `9c8431e`, `ea0fef0`, `5f22179` 커밋 내용 | PR 후보별 독립성, 범위, blocker 판단 |
| `D:/vibe-coding/rhwp/tools/install_rhwp_openwith.ps1` | installer 부작용 확인 |
| `D:/vibe-coding/rhwp/tools/rhwp-launcher/main.go` | 런처가 `rhwp_launcher.ps1`에 의존함을 확인 |
| `D:/vibe-coding/rhwp/rhwp-studio/package.json` | `preview` 스크립트 변경 범위 확인 |
