# 구현계획서 — Task #1217: Dependabot 결합 의존성 그룹화

- **이슈**: #1217 / **브랜치**: `local/task1217`
- **수행계획서**: `task_m100_1217.md`
- **대상 파일**: `.github/dependabot.yml` (단일 설정 파일)
- **작성일**: 2026-06-01

## 단계 구성 (3단계)

### Stage 1 — `.github/dependabot.yml` 그룹 추가

기존 구조(version 2, cargo + npm 두 블록, target-branch=devel, weekly, limit 10)를 보존하고
각 ecosystem 블록에 `groups` 키만 추가한다.

**cargo 블록** — `resvg-stack` 그룹:
```yaml
    groups:
      resvg-stack:
        patterns:
          - "svg2pdf"
          - "pdf-writer"
          - "usvg"
          - "resvg"
          - "ttf-parser"
```

**npm 블록** — `vite-stack` 그룹:
```yaml
    groups:
      vite-stack:
        patterns:
          - "vite"
          - "vite-plugin-pwa"
```

설계 결정:
- `applies-to` 미지정 → 기본값(version-updates)에 적용. security-updates 는 그룹화하지 않음
  (보안 패치는 개별 신속 처리가 안전 — Dependabot 기본 동작 유지).
- `update-types` 미지정 → major/minor/patch 모두 그룹에 포함. 결합 크레이트는 patch 도
  버전 동반이 안전.
- 그룹 외 crate 는 자동으로 개별 PR 유지(그룹은 매칭된 것만 묶음).

### Stage 2 — 검증

1. YAML 문법: `python3 -c "import yaml,sys; d=yaml.safe_load(open('.github/dependabot.yml')); print('OK', list(d.keys()))"`.
2. 구조 확인: 두 ecosystem 블록 각각 groups 존재, 기존 키(target-branch/schedule/limit) 보존.
3. Dependabot groups 스펙 대조: `patterns` 키 형식, 그룹명 영숫자/하이픈 규칙 준수.
4. 소스/Cargo.toml/package.json diff 없음 확인 → 빌드·테스트 무영향(자명, 실행 불요).

### Stage 3 — 커밋 + 최종 보고서

1. 단계별 보고서 `working/task_m100_1217_stage1.md`, `_stage2.md` 작성.
2. 최종 보고서 `report/task_m100_1217_report.md` 작성(사후 관찰 항목 포함:
   다음 주간 실행 시 그룹 PR 묶임 확인은 머지 후 GitHub 동작 관찰).
3. 오늘할일 `orders/20260601.md` 갱신.
4. `.github/dependabot.yml` + 문서 함께 `local/task1217` 커밋
   (`feedback_commit_reports_in_branch`). merge 전 `git status` 확인.

## 비고

- 설정 변경만이므로 단계가 가볍다(3단계). 각 단계 후 승인 게이트는 유지.
- 그룹 적용의 실효(단독 #1215 재발 방지)는 머지 후 다음 weekly 실행에서 관찰 — 보고서에 명시.
