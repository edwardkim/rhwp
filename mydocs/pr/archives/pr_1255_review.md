# PR #1255 검토 - Issue #1253 미주 모양 UI와 HWPX 구분선 굵기 정합

- **작성일**: 2026-06-03
- **PR**: #1255 (OPEN)
- **제목**: `task 1253: 미주 모양 UI와 HWPX 구분선 굵기 정합`
- **컨트리뷰터**: @jangster77
- **연결 이슈**: #1253
- **base/head**: `devel` <- `task_m100_1253`
- **Head SHA**: `9080d94560269e8622a988d12ca6cb81defa7649`
- **현재 devel**: `5453b254`
- **규모**: 11 files, +689 / -26
- **GitHub 상태**: `MERGEABLE`
- **CI**: `Build & Test`, `CodeQL`, `Canvas visual diff` 통과. `WASM Build`는 workflow 조건상 skipping.
- **PR 댓글/리뷰**: 없음

## 1. 번호 확인

작업지시자는 “#1253 PR”이라고 요청했지만, GitHub에서 #1253은 이슈다.

실제 처리 대상 PR은 다음이다.

- Issue #1253: `미주 모양 UI와 미주 사이/구분선 아래 렌더링이 한컴과 불일치`
- PR #1255: `task 1253: 미주 모양 UI와 HWPX 구분선 굵기 정합`

따라서 본 검토는 PR #1255를 Issue #1253 처리 PR로 본다.

## 2. PR 요약

PR #1255는 미주 모양 관련 문제를 두 갈래로 처리한다.

1. rhwp-studio `주석 모양 > 미주 모양` UI를 한컴오피스 2024 대화상자에 더 가깝게 보강한다.
2. HWPX `<hp:noteLine width>` 파서가 미주/각주 구분선 굵기를 일반 선 굵기 코드표로 해석하도록 수정한다.

PR 본문에서 명시하듯, `3-09월_교육_통합_2022.hwp` 9쪽의 실제 미주 조판 차이는 아직 후속 Stage3 문제로 남아 있다. 이번 PR은 UI/API/HWPX 파서 정합을 수용 범위로 둔다.

## 3. 주요 변경

| 파일 | 변경 |
|---|---|
| `rhwp-studio/src/ui/endnote-shape-dialog.ts` | 구분선 종류/굵기/색 선택 UI를 미리보기 버튼과 팝업 메뉴로 변경 |
| `rhwp-studio/src/ui/endnote-shape-dialog.ts` | 구분선 길이 항목에 `사용자` 콤보 추가 |
| `src/parser/hwpx/section.rs` | `noteLine width`를 `parse_hwpx_line_width()`로 파싱 |
| `src/parser/hwpx/section.rs` | `0.12mm -> 1`, `0.7mm -> 9` 회귀 검증 추가 |
| `mydocs/*task_m100_1253*` | 계획, 단계 기록, 최종 보고서 추가 |

## 4. 타당한 부분

### 4.1 UI 정합 개선 범위가 명확하다

기존 UI는 구분선 종류와 굵기를 텍스트 select로만 보여줬기 때문에, 한컴 대화상자 기준 설정과 같은지 눈으로 판단하기 어려웠다.

이번 PR은 선 종류/굵기/색을 미리보기 중심으로 바꿔 사용자가 한컴 설정과 비교하기 쉽게 만든다. 저장 계약은 그대로 두고 표시만 보강한 점도 안전하다.

### 4.2 HWPX 선 굵기 파서 수정은 기존 코드표와 맞다

기존 HWPX `noteLine width` 파서는 `mm * 10` 방식이라 `0.7mm`가 raw `7`로 들어갈 수 있었다.

프로젝트의 일반 선 굵기 코드표 기준으로는 `0.7mm -> 9`가 맞고, PR은 `parse_hwpx_line_width()`를 사용해 이 경로를 공통화한다. 추가된 테스트도 이 의도를 직접 고정한다.

### 4.3 내부 미주 간격 저장 계약을 건드리지 않았다

PR은 다음 계약을 유지한다.

- Studio JSON `noteSpacing` -> 내부 `raw_unknown` -> 한컴 UI `미주 사이`
- Studio JSON `separatorMarginBottom` -> 내부 `note_spacing` -> 한컴 UI `구분선 아래`

최근 미주 조판 관련 PR들이 이어진 상태라, 이번 PR에서 저장 슬롯을 바꾸지 않은 것은 회귀 위험을 낮춘다.

## 5. 주의 사항

### 5.1 실제 9쪽 미주 조판 차이는 아직 해결 범위가 아니다

PR 본문과 Stage3 기록 모두 `3-09월_교육_통합_2022.hwp` 9쪽의 실제 미주 VPOS/LINE_SEG 소비 차이를 후속 문제로 남긴다.

따라서 이번 PR의 시각 판정은 “미주 모양 UI와 HWPX 구분선 굵기 파서” 중심으로 해야 한다. 9쪽 조판 차이를 이유로 본 PR을 막기보다는, 별도 이슈/후속 단계로 추적하는 편이 맞다.

### 5.2 PR base가 현재 devel보다 뒤처져 있다

PR head는 `9080d945`, base SHA는 `3bd66137`이며 현재 `devel`은 `5453b254`다.

로컬에서 `git merge-tree devel pr/1255`를 실행한 결과 충돌 없는 merge tree가 생성되었다. 그래도 실제 반영 전에는 최신 `devel` 위 검증 브랜치를 만들어 병합 후 테스트해야 한다.

### 5.3 UI 팝업 구현은 추후 공통 컴포넌트화 후보

`endnote-shape-dialog.ts`에 선 미리보기 팝업과 색상 팔레트 구현이 직접 들어갔다.

이번 범위에서는 적절하지만, 다른 대화상자에서도 같은 선/색 선택 UI가 필요해지면 공통 컨트롤로 분리하는 편이 좋다.

## 6. 자동 확인

GitHub checks:

| 항목 | 결과 |
|---|---|
| `Build & Test` | 통과 |
| `Canvas visual diff` | 통과 |
| `CodeQL` | 통과 |
| `Analyze (rust)` | 통과 |
| `Analyze (javascript-typescript)` | 통과 |
| `Analyze (python)` | 통과 |
| `WASM Build` | skipping |

로컬 사전 확인:

```text
git merge-tree devel pr/1255
```

결과: 충돌 없음.

## 7. 권장 검증

최신 `devel` 기준 검증 브랜치를 만들고 PR #1255를 병합한 뒤 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
npm run build --prefix rhwp-studio
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
```

시각/동작 판정 후보:

| 대상 | 확인 항목 |
|---|---|
| rhwp-studio `주석 모양 > 미주 모양` | 구분선 종류/굵기/색 UI가 한컴 기준에 가깝게 표시되는지 |
| `samples/3-09월_교육_통합_2022.hwp` | 미주 모양 대화상자 설정값이 기존보다 식별 가능해졌는지 |
| HWPX 미주 샘플 | `noteLine width="0.7 mm"`가 raw `9` 굵기로 렌더/저장 경로에 들어가는지 |

## 8. 권장 처리

권장안: **수용 후보로 진행한다.**

근거:

- PR 범위가 UI 정합과 HWPX 선 굵기 파서 보정으로 명확하다.
- 저장 슬롯 계약을 바꾸지 않아 기존 미주 간격 회귀 위험이 낮다.
- GitHub CI가 통과했다.
- 최신 `devel` 위 병합 충돌이 없다.
- 실제 9쪽 미주 조판 차이는 PR 본문에서 후속 문제로 분리되어 있어, 이번 PR 수용 판단과 분리할 수 있다.

단, 최종 반영 전에는 최신 `devel` 위에서 검증 브랜치를 만들고 WASM/Studio 빌드 및 메인테이너 UI 시각 판정을 진행해야 한다.

## 9. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1255-verify` 브랜치를 현재 `devel`에서 생성
2. PR #1255를 병합
3. Rust/Studio/WASM 검증 실행
4. rhwp-studio 미주 모양 UI 메인테이너 시각 판정
5. 판정 통과 후 devel 반영 및 PR/이슈 후속 처리
```
