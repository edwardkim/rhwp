# #9097: 최신 devel 반영과 제출 전 검증

## 기준과 영향 범위

- 작업 브랜치: `task/9097-frontend-canvas-ci`.
- 이전 기준: `1805d8f08cb3a54bccda8dfa6647432b7799f074`.
- 새 기준: `4e0b20d2922953ee7ab9a1716ac819b921fe484b`.
- 로컬 검증 코드: `1cf5b6d9ecff01d4fb0a30da632fb77e54ce120b`.
- [구현 단계 기록](task_m100_9097_stage1.md),
  [최종 로컬 검증과 입력 hash](assets/issue_9097/local-validation.json).

기준 갱신으로 PR #9096의 parser·CLI·Print/font·배치 보정과 검토 기록이 반영됐다.
리베이스는 충돌 없이 완료됐다. 후보가 새 기준에 추가하는 변경에는 제품 Rust·Studio 소스,
실물 입력·PDF·baseline·시각 허용치·글꼴이 없다. 해당 PR의 parser·저장·CLI·Print 구현과
그 소비 경로는 새 기준 그대로이며 이번 변경의 Native PDF 생략 조건도 바꾸지 않는다.
이는 CI 실행 경로의 변경이므로 제품 조판 수정이나 신규 시각 기대값 등록으로 취급하지 않는다.
Native 전체 회귀·편람 TSV의 신규 실행은 이 CI 변경의 로컬 검증 범위에 포함하지 않았다.

엔진 코드가 갱신됐으므로 기존 WASM의 통과 결과만 재사용하지 않고, 새 기준을 포함한 코드
head에서 standard release WASM을 다시 빌드하고 실제 브라우저 경로를 다시 실행했다.
기존 검사·허용치를 유지한 채 #9077 선택 동작, legacy/layer 출력과 CanvasKit 준비 상태가
모두 유지됐다. 최신 devel과 후보의 제품·입력 파일은 동일하다.

## 실행 결과

| 검사 | 실제 결과 |
| --- | --- |
| classifier·policy·trusted reuse·artifact 계약 | Node 343/343 PASS, skip 0 |
| `test_*workflow.py` 전체 | Python 250/250 PASS |
| 변경 workflow 3개의 actionlint 1.7.12 | PASS |
| 생성 WASM 선언·editor embed 계약 | 3/3 PASS |
| standard release, 실제 wasm-opt 실행 | exit 0, 총 1641.8초; Rust 컴파일 23분 42초 |
| 공용 browser 실행기 | exit 0, 175.05초 |
| #9077 setter와 원시 setter 대조 | 12개 PASS |
| legacy/layer 기존 기본 문서 첫 쪽 | 3개 PASS; KTX 0.00617%, 나머지 0% |
| CanvasKit readiness | 8/8 PASS, 실패·누락 0 |
| 실제 브라우저 WASM 응답 hash | 18개 모두 이번 빌드와 일치 |
| 사용한 HWP/HWPX 입력 | 11개 모두 검증 코드 commit과 byte/hash 일치 |
| `git merge-tree --write-tree upstream/devel HEAD` | exit 0, 충돌 없음 |

WASM hash는 `ee2c1a26789167c69c405dcfed268e38bfe1bf7734280cd46e4d364ab0ba459f`다.
이전 기준의 `16e3f571…`와 구별되는 산출물을 브라우저가 실제로 수신했다.
Windows Chrome 157을 사용한 로컬 결과이며 CI의 고정 Chromium 실행 시간으로 일반화하지 않는다.

Vite의 `/samples` middleware는 저장소 `samples/`를 우선 소비한다. 실행기가 계산한
document digest와 실제 파일·Git commit을 대조해 위 입력 기록에 보존했다.
기존 커밋 파일을 재사용하므로 입력 사본은 추가하지 않았다. PDF 대조나 한컴 출력과의
일치율을 새로 측정했다고 주장하지 않는다.

Windows 검증용 `public/fonts` 임시 연결과 생성 public 선언 파일을 원복했다.
`assets/fonts`, `ttfs`, `rhwp-studio/public/`의 제품 파일 변경은 없으며 글꼴은 커밋하지 않는다.

## 제출과 원격 관찰 조건

본인 열린 PR 조회 결과는 0건이었으며 리뷰어 예외·기존 PR 간 시뮬레이션 대상도 0건이다.
제출 후보와 최신 기준의 병합 시뮬레이션, 링크와 diff 검사 후 일반 Open PR로 제출한다.
devel branch API에서 required context `Build & Test`를 확인했다. 관리자 상세 protection
endpoint는 404를 반환했으므로 추가 권한이나 bypass 가능 여부를 추정하지 않는다.

이번 PR 자체는 CI 실행 정의·분류 정책 변경이므로 Full CI를 확인한다. frontend 전용 선택
경로의 분류·실행 계약은 로컬 검사로 확인했으며, 최초 정책 PR의 전체 실행 시간을 그 경로의
실제 비용 개선으로 기록하지 않는다. required context뿐 아니라 해당 head의 관련 workflow와
실제 Canvas 빌드·브라우저 step이 완료됐는지 확인한다. 실패 시 보정·재검증하며 조건을 완화하지 않는다.

사용자가 지정한 옵션 2에 따라 원 코드 PR의 검토·CI·병합 후 별도 기록 PR에서 review·오늘할일을
반영한다. rollback이 필요하면 해당 구현 commit을 되돌리는 일반 PR을 사용하며 devel 직접
push나 branch protection 우회는 하지 않는다.
