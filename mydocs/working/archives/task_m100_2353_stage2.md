# #2353 2단계 — e2e 관리 체계 설계안 (승인 게이트)

- 계획서: `mydocs/plans/task_m100_2353.md` (구현계획서 대행 단계)
- 입력: 1단계 인벤토리 76개 + 관찰 4건 + 공허 통과 발견

## 1. Manifest — 위치·형식

**권고: `rhwp-studio/e2e/MANIFEST.md`** (후보 A)

| 기준 | A. e2e/MANIFEST.md | B. mydocs/manual/ 하위 |
|------|--------------------|------------------------|
| 갱신 근접성 | **파일 옆** — 테스트 추가 PR 의 diff 에 함께 보임 | 코드와 분리 — 드리프트 재발 위험 (매뉴얼 18/76 이 그 증거) |
| 검사 스크립트 파싱 | 경로 고정·형식 통제 용이 | 동일 가능하나 이점 없음 |
| 문서 거버넌스 | mydocs 범위 밖 (front matter 의무 없음) | front matter 정합 |

- 형식: 고정 컬럼 markdown 표 — `파일 | 분류 | 용도 | 대상 | 샘플 | 배선 | 상태`.
  분류·상태는 열거값 고정 (검사 스크립트가 파싱·검증).
- 분류 열거: `상시` / `진단` / `유틸`. 상태 열거: `active` / `hold`(보류 이슈
  종속) / `deprecated`(폐기 예고 — 사유·날짜 병기).
- mydocs 쪽에는 e2e-cdp.md 가 실행 가이드로 존치하고, 목록 절(2.2)은
  MANIFEST 참조 한 줄로 대체.

## 2. 명명 규칙 (신규부터 적용, 기존은 보수적)

| 유형 | 규칙 | 예 |
|------|------|-----|
| 상시 회귀 | `<도메인>[-issue<N>].test.mjs` — `.test.mjs` 필수 | `undo-contracts.test.mjs` |
| 일회성 진단/프로브 | `probe-*.mjs` — `.test` 금지 (러너·집계 제외 명시) | `probe-issue2021-cell-timing.mjs` |
| 수동 디버그 | `debug-*.mjs` — `.test` 금지 | `debug-pagination.mjs` |
| 유틸/러너 | 접미 없는 `.mjs` | `helpers.mjs`, `run-render-diff.mjs` |

- `.check.mjs` 관례 폐지 (probe- 또는 .test 로 흡수).
- **기존 파일 개명은 이번에 4건만**: `debug-*.test.mjs` 3건 → `debug-*.mjs`
  (배선·문서 참조 0건 확인됨), `issue-2021-probe.mjs` → 폐기(아래). 나머지
  혼재(issue-N 접두 유무 등)는 manifest 가 실명으로 관리하므로 일괄 개명
  하지 않음 — 신규부터 규칙 적용.

## 3. 생명주기 절차

- **추가**: MANIFEST 행 등재 의무 — 미등재 시 대조 검사 FAIL (원장 트립와이어
  방식). 배선(npm script/CI) 여부도 행에 기록.
- **변경**: 용도·샘플·배선 변경 시 해당 행 갱신 (검사가 배선 실재를 검증).
- **폐기**: 목적 달성한 일회성 스크립트는 **삭제** — git history 가 archive
  (stub 없음, #2313 선례). manifest 에서 행 제거. 보류 이슈 종속 진단은
  상태 `hold` 로 존치.
- **이번 폐기 제안 (승인 대상)**: ①`issue-2021-probe.mjs` (#2021 해소 완료된
  계측 프로브) ②`task1315-load.check.mjs` (헤더에 "임시 검증" 명시, #1315
  4단계 종료) ③`kps-ai-host.test.mjs` (kps-ai 의 host 변형 — helpers 의
  `--mode=host` 로 대체 가능 확인). 그 외 보류 진단 2건(body-outside/
  grid-mode)은 보류 이슈 미결이므로 `hold` 존치.

## 4. 드리프트 대조 검사 — `scripts/check_e2e_manifest.py`

1. **양방향 대조**: e2e/*.mjs ↔ MANIFEST 행 — 미등재 파일 FAIL / 유령 행 FAIL
2. **명명 정합**: 분류 `상시` 행은 `.test.mjs`, `진단` 은 probe-/debug- 접두
   (기존 예외는 manifest `비고` 의 `legacy-name` 표기로 면제 — 신규만 강제)
3. **배선 실재 검증**: package.json e2e script·워크플로가 가리키는 파일 존재
   확인 — **공허 통과(없는 파일명 실행) 재발 방지** (1단계 발견 반영)
4. 로컬 실행 원칙 (CI 미편입 — check_markdown_links 와 동일 운용)

## 5. 구현 범위 (3단계)

MANIFEST 전수 작성(76−폐기 3=73행) + 검사 스크립트 + e2e-cdp.md 목록 절
대체 + 개명 3건·폐기 3건 실행 + npm script `e2e:manifest-check` 등록(선택).

— 본 설계안 승인 시 3단계 구현에 착수합니다. 폐기 3건은 개별 항목으로
승인 여부를 명시해 주셔도 됩니다 (기본: 3건 모두 폐기).
