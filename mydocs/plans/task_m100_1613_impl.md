# Task M100 #1613 구현계획서 — 저장 출력 포맷(HWP/HWPX) 선택 메뉴

- 이슈: #1613
- 브랜치: `local/task1613`
- 작성일: 2026-06-28
- 수행계획서: `mydocs/plans/task_m100_1613.md`

## 구현 개요

`file:save-as` 의 export·blob·FS 저장 로직을 **출력 포맷(isHwpx)을 인자로 받는 공유 헬퍼**로
추출한다. 기존 `file:save-as`(출처 포맷)와 신규 `file:save-as-hwp`/`file:save-as-hwpx`(명시
포맷)가 이 헬퍼를 호출한다. WASM/Rust 변경 없음. index.html 메뉴 항목 추가.

핵심: 포맷 결정만 출처(`getSourceFormat`)에서 인자로 바꾸면, export(`exportHwp`/`exportHwpx`)·
파일명 확장자·MIME 가 자동으로 따라간다(기존 `saveFileNameFor(name, isHwpx)` 재사용).

---

## 1단계 — 포맷 인자 저장 헬퍼 추출 + 명시 저장 명령

**대상**: `rhwp-studio/src/command/commands/file.ts`

- 공유 헬퍼 추가:
  ```ts
  // 출력 포맷(isHwpx)을 명시 받아 export → blob → FS 저장(picker) → 폴백 download.
  async function saveAsFormat(services, isHwpx: boolean): Promise<void> {
    const saveName = saveFileNameFor(services.wasm.fileName, isHwpx);
    const bytes = isHwpx ? services.wasm.exportHwpx() : services.wasm.exportHwp();
    const blob = new Blob([...], { type: isHwpx ? 'application/hwp+zip' : 'application/x-hwp' });
    // 기존 file:save-as 의 FS Access + 폴백 흐름 동일
  }
  ```
- 기존 `file:save-as` execute 를 `saveAsFormat(services, getSourceFormat()==='hwpx')` 호출로 정리
  (동작 동일, 회귀 없음).
- 신규 명령 2개:
  - `file:save-as-hwp` (label "HWP 형식으로 저장") → `saveAsFormat(services, false)`.
  - `file:save-as-hwpx` (label "HWPX 형식으로 저장") → `saveAsFormat(services, true)`.
  - `canExecute: ctx.hasDocument`.

**완료 기준**: tsc 에러 0. 기존 file:save-as 동작 보존.

## 2단계 — 메뉴 항목 추가 + 단축키/라벨

**대상**: `rhwp-studio/index.html`, (필요 시) `menu-shortcut-labels.ts`

- 파일 메뉴(`data-menu="file"`)에 항목 추가(다른 이름으로 저장 아래):
  ```html
  <div class="md-item" data-cmd="file:save-as-hwp"><span class="md-icon"></span><span class="md-label">HWP 형식으로 저장...</span></div>
  <div class="md-item" data-cmd="file:save-as-hwpx"><span class="md-icon"></span><span class="md-label">HWPX 형식으로 저장...</span></div>
  ```
- 문서 없을 때 비활성(`disabled`) 토글이 기존 file:save 처럼 동작하는지 확인(menu-bar.ts 의 enable
  로직 적용 범위 점검).
- 단축키는 신설하지 않음(메뉴 클릭 우선). 필요 시 후속.

**완료 기준**: 메뉴에 두 항목 노출, 문서 열림 시 활성화.

## 3단계 — 검증 + 보고서

- studio tsc 에러 0, `npm test` 통과.
- 수동/e2e: HWP 문서 → HWPX 저장(.hwpx, PK 매직), HWPX 문서 → HWP 저장(.hwp). 기본 저장 회귀 없음.
  - 가능하면 e2e 테스트(`hwpx-direct-save.test.mjs` 패턴) 추가로 포맷 명시 저장 가드.
- 최종 보고서 + 오늘할일(#1613) 갱신.

**완료 기준**: tsc/test 통과 + 보고서/오늘할일 커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `rhwp-studio/src/command/commands/file.ts` | saveAsFormat 헬퍼 + 명령 2개 (~40줄) |
| `rhwp-studio/index.html` | 파일 메뉴 항목 2개 |
| `rhwp-studio/e2e/...` | (선택) 포맷 명시 저장 e2e |
| `mydocs/working/task_m100_1613_stage{N}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_1613_report.md` | 최종 보고서 |

## 위험 / 주의

- 메뉴 활성/비활성 토글: menu-bar.ts 가 `file:save` 만 특별 처리하는지(line 141) 확인하여 신규
  항목도 문서 유무에 따라 토글되게 한다.
- HWP↔HWPX 변환 충실도는 본 작업 범위 밖(기존 export API 사용). 한컴 호환은 작업지시자 시각 판정.
- 라벨/네이밍("HWP 형식으로 저장")은 한컴 "다른 이름으로 저장"의 형식 선택과 동등 의미로 통일.
