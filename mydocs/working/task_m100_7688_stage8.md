---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 8단계 — PR #7697 CI 실패의 검사 계약 보완

사용자가 두 실패의 해결을 승인했다. 실패 head는
`dadc7be0630b068093f02f4d6d3382adec736340`, 현재 devel은
`79abd9e49675f4286eaed38197f7e837afb744e7`다.
제품 Rust/TS, HWP/HWPX/PDF 및 글꼴·WASM bytes는 변경하지 않는다.

## Render Diff의 검사 계약

[실패 실행](https://github.com/edwardkim/rhwp/actions/runs/37913352921)은 이미지 비교 전에
`renderer-contract.test.mjs`에서 중단했다. 전체 검사 파일을 로컬에서 실행하니
먼저 가려졌던 화살촉/옛한글 fixture 계약의 불일치도 확인됐다.

| 기존 조건 | 현재 제품 계약과 교정 |
| --- | --- |
| local font 준비의 인자가 `report.requiredFontFamilies`여야 한다 | explicit 선택에서는 report가 null일 수 있으므로 재조회한 `fontRequirements.requiredFontFamilies`를 사용한다. 실제 준비·실패 fallback은 기존 브라우저 E2E로 확인한다. |
| ArrowStyle만 있는 line op에서 CanvasKit이 화살촉을 재계산한다 | 생산자가 `arrowHeads`의 path를 직렬화한다. 실행 fixture에 commands를 공급하고 tip·전체 path의 동일 소비 및 형상 누락 진단을 검사한다. |
| backend에 각 ArrowStyle 이름별 형상 분기가 존재한다 | 공통 형상을 `createCommandPath`로 소비하고 fill을 보존한다. backend별 형상 추측을 요구하지 않는다. |
| bundled alias map의 값이 단일 face object다 | 여러 style을 보존하는 face 배열이다. 옛한글의 shaping manager 없는 face 거부와 유효한 단일 face의 도달성 assertion을 유지한다. |

검사 조건을 삭제하거나 허용치를 낮추지 않는다. 직렬화 path는 대조 입력이며 한컴의 특정
화살촉 크기를 새로 주장하지 않는다. 제품의 실제 크기·방향 증거는 [5단계](task_m100_7688_stage5.md)에 있다.

## 새 sample의 text-overlap 원장

Archive D의 두 실패는 각각 새 `control-2.hwpx`, `control-3.hwpx`의 5건이 baseline에 없어서 발생했다.
fail-fast로 다른 partition은 종료됐으므로 두 문서만 고치지 않고 파생 sample 5개를 모두 대조했다.

`group-drawing-02.hwp` 및 파생 5개에 동일 `layout-anomaly --json`을 실행했다.
수정 전 devel `6f66932a…`의 보존 바이너리와 최종 바이너리에서 각 5건이며,
모든 anomaly의 쪽·소유 path·bbox·겹침 폭/높이가 동일했다. 최신 devel까지 제품 `src/` 변경은 없다.
기존 원문의 원장도 5건이다. 파생 입력은 저장 줄·글상자·그룹을 유지하고 화살촉 속성만 바꿨다.

| 파생 입력 | 수정 전 → 후 text-overlap | Native/fresh WASM 전체 1쪽 최저 |
| --- | --- | --- |
| control-1.hwpx | 5 → 5 | 99.87570% |
| control-2.hwpx | 5 → 5 | 99.87593% |
| control-3.hwpx | 5 → 5 | 99.84531% |
| control-short.hwpx | 5 → 5 | 99.87554% |
| control-compound.hwpx | 5 → 5 | 99.87578% |

[입력/PDF 해시](../../samples/issue7688/README.md)와
[신규 fixture 등록 절차](../manual/pr_review/local_validation.md#431-새-hwphwpx-fixture의-baseline-등록--코퍼스-래칫-일곱)에 따라
이 5개만 신규 행으로 등록한다. 기존 문서의 기준·검출 함수·실패 조건은 유지한다.
검출된 것은 작은 글상자의 `item id`가 두 줄로 나뉘어 뒤 글상자의 `item`과 교차하는 기존 차이다.
독립 Print와 실제 PNG를 직접 대조했다. 높은 전체 점수가 이 차이의 해소를 뜻하지 않는다.
이번 PR은 해당 글상자 줄바꿈/clip 결함을 해결했다고 주장하지 않는다.

수정 전 CLI SHA-256: `baf5979778da39292c0ac4d5521f411c500928c5862e76b67535eefee583bf9a`.
수정 후 CLI SHA-256: `c5ad2a3d672bcf500b86252bafda95894bfb8cd5c2539e89be61fd774def09ea`.
JSON·전후 로그는 ignored `output/pr-review/renderer-backend-audit-20261009/final/ci-fix/`에 보존한다.

## 검증 상태

Render contract 교정과 실제 브라우저 font 준비·style·실패 fallback 검사는 통과했다.
원장 수정 전 전체 partition은 11 PASS / 5 FAIL로 다섯 신규 입력의 미등록을 재현했다.
Studio 단위 검사 1,825 PASS / 0 FAIL / 2 skipped, TypeScript noEmit 및 E2E manifest도 통과했다.
Render Diff의 CI 후속 단계는 로컬에서 Canvas 3쪽 모두 PASS, 직접 PDF 호환성 3/3 PASS다.
PDF 보조 보고서의 4 warnings / 0 errors는 직접 PDF gate 실패와 구별한다.
수정 후 코퍼스 래칫은 84 PASS / 0 FAIL이며 text-overlap 16개 partition도 전부 통과했다.
코퍼스 선택 밖 10,519개는 filter로 skipped이며 전체 Rust 재실행으로 보고하지 않는다.
Clipping controlset은 변경하지 않았고 신규 파생 입력은 그 controlset에 없어 해당 래칫은 비해당이다.
Rust source/test helper는 변경하지 않아 이미 통과한 동일 bytes의 Rust lint와 Docker WASM을 재사용했다.
공용 target은 `/home/edward/mygithub/rhwp/target/pr-review`, nextest thread 수는 4다.
최종 로컬 검증·새 head CI의 성공 전에는 self-review·오늘할일 trailing 기록이나 merge로 진행하지 않는다.
