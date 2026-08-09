# Task M100 #4272 Stage 3 — 3중 중첩 표 객체 복사

- 이슈: [#4272](https://github.com/edwardkim/rhwp/issues/4272)
- 선행 단계: [Stage 2 — 선택 텍스트 복사·붙여넣기](task_m100_4272_stage2.md)
- 기준 commit: `a7be7c0ac` (Stage 2 구현)
- 작업 브랜치: `fix/issue-4272-nested-cell-text-selection`
- 작성일: 2026-08-09 KST
- 상태: 구현·전체 PR 검증·후속 결함 분리 완료, PR #4276 self-review 기록 준비

## 목표

표 안의 표 안의 표를 객체 선택한 뒤 Ctrl+C 했을 때 선택된 자식 표를 native 내부 클립보드와
시스템 클립보드용 HTML에 정확히 복사한다. 객체 선택 렌더링과 페이지네이션은 변경하지 않는다.

## RED 재현

- 샘플: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 위치: 물리 5쪽 `구 분` 셀에서 Escape로 선택한 깊이 3 자식 표
- 객체 선택 참조는 다음 전체 경로를 정확히 보존했다.

  ```text
  [(1,0,0), (2,0,12), (0,0,0)]
  ```

- 기존 Ctrl+C는 `copyControl(0, 7, 1, fullPath)`를 호출했다. native는 full path가 가리키는
  선택 표 내부 셀 문단까지 내려간 다음 바깥 호환 인덱스 `1`을 조회해
  `렌더링 오류: 컨트롤 1 범위 초과`를 반환했다.
- 결과는 시스템 클립보드 빈 문자열, 내부 클립보드 없음, 표 control 없음이었다.

RED 상태는 [재현 JSON](../../output/4272/nested-table-object-copy-red.json)에 보존했다.

## 원인과 수정

객체 선택용 `cellPath`와 clipboard API의 주소 계약이 다르다.

- 선택·렌더링용 전체 path: 선택된 표의 셀까지 포함한다.
- 복사용 owner path: 선택된 표 control을 소유한 문단까지만 가리킨다.
- 선택된 표 control index: 전체 path 마지막 엔트리의 `controlIndex`다.

`tableObjectClipboardTarget()` 순수 헬퍼가 이 변환을 한 곳에서 수행한다. 실제 경로는 다음처럼
변환된다.

```text
full path        [(1,0,0), (2,0,12), (0,0,0)]
owner path       [(1,0,0), (2,0,12)]
control index    0
```

키보드 Ctrl+C와 컨텍스트 메뉴·도구 상자의 `performCopy()`가 같은 헬퍼를 사용한다. 본문 표는
기존 `ref.ci`와 빈 owner path를 그대로 사용한다. 중첩 표 잘라내기·삭제는 기존 미지원 게이트를
유지해 작업 범위를 확장하지 않았다.

이 계산은 사용자가 복사 명령을 내릴 때 path 깊이만큼 한 번 실행된다. 마우스 드래그, 렌더링,
`requestAnimationFrame`, 페이지네이션 hot path에는 새 작업을 추가하지 않았다.

## 래칫

- Studio 순수 함수 테스트
  - 깊이 3 참조 → owner path depth 2, control index 0
  - 본문 표 → 빈 owner path, 기존 `ref.ci`
- Studio source guard
  - 키보드 Ctrl+C와 `performCopy()`가 같은 변환 결과를 `copyControl`과
    `exportControlHtml`에 전달
- 실제 fixture Rust 래칫
  - `(section 0, paragraph 7, owner path [(1,0,0),(2,0,12)], control 0)` 복사
  - 결과 `[표]`, 내부 클립보드 존재, control 포함
- 실제 browser CDP E2E
  - 물리 5쪽 `구 분` hit → Escape 표 객체 선택 → Ctrl+C
  - 전체 선택 path는 유지하면서 clipboard 호출만 owner 주소로 변환
  - plain text와 HTML 표 export 및 내부 control을 함께 검증

## 검증

| 검증 | 결과 |
|---|---|
| 실제 샘플 Rust #4272 통합 래칫 | 2/2 통과 |
| Studio 표 객체 주소 변환 focused 테스트 | 3/3 통과 |
| Studio 전체 `npm test` (샌드박스 밖) | 819/819 통과 |
| TypeScript `tsc --noEmit` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| 호스트 Chrome CDP #4272 3중 표 객체 Ctrl+C | 통과 |
| 호스트 Chrome CDP #4252 인접 중첩 표 객체 선택 | 통과 |
| 호스트 Chrome CDP #4272 물리 5쪽 텍스트 Ctrl+C/V | 통과 |
| 호스트 Chrome CDP #4272 물리 11쪽 문단 22 Ctrl+C | 통과 |

Studio 전체 테스트는 알려진 `spawnSync()` 샌드박스 `EPERM` 오탐을 피하도록 처음부터 샌드박스 밖에서
실행했다.

소스 변경은 Studio의 복사 주소 변환뿐이며 Rust/WASM 공개 API와 바이너리는 변경하지 않았다. 따라서
Stage 2에서 이미 dev 서버에 배치한 표준 Docker WASM을 그대로 사용했고 중복 WASM 빌드는 수행하지
않았다. 실행 중인 Vite dev 서버에는 TypeScript 변경이 hot reload로 반영됐다.

## 전체 PR 후보 검증

작업지시자 승인 뒤 최신 `upstream/devel` `e4d07fab7`을 task branch에 병합했다. 새 devel의
passthrough invalidation guard가 경로 기반 복사 API를 분류하지 않아 전체 nextest 최초 실행에서
`classification_drift_is_blocked` 1건이 실패했다. 이 API는 문서를 읽고 세션의
`self.clipboard`에만 기록하므로 기존 복사 API와 동일하게 `Exempt::SessionState`로 명시했다.
focused guard 5/5와 전체 nextest 재실행으로 분류가 정확함을 확인했다.

최종 후보 `e68cab959` 기준 결과는 다음과 같다.

| 검증 | 최종 결과 |
|---|---|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 3,361 통과, 13 ignored |
| 전체 `cargo nextest` | 5,484/5,484 통과, 35 skipped |
| native Skia lib | 58/58 통과 |
| native Skia 누락 그림 fixture | 2/2 통과 |
| native Skia direct PDF | 4/4 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| doctest | 4 통과, 2 ignored |
| Studio `npx tsc --noEmit` | 통과 |
| Studio 전체 `npm test` (샌드박스 밖) | 819/819 통과 |
| 표준 Docker WASM 빌드 | 통과, `pkg/rhwp_bg.wasm` 갱신 |
| headless #4272 텍스트 선택·복사·붙여넣기 | 통과 |
| headless #4272 물리 11쪽 텍스트 복사 | 통과 |
| headless #4272 3중 표 객체 복사 | 통과 |

최종 WASM SHA-256은
`78658b33f55919407295491c72678bbdf2327968f0a700404a50b08d10b226ca`다. 3중 표 객체 E2E의
첫 실행은 테스트 단언 전에 headless Chrome 프로세스가 시작되지 않아 종료됐고, 동일 명령 재실행은
모든 단언과 브라우저 warning/error 0건 조건까지 통과했다.

## CDP 시각·상태 증적

- [3중 표 객체 복사 상태 JSON](../../output/4272/nested-table-object-copy.json)
- [3중 표 객체 선택 PNG](../../output/4272/nested-table-object-copy.png)
- [HTML 보고서](../../output/e2e/issue-4272-nested-table-object-copy-report.html)

관측 결과:

- 선택 참조: `sec=0`, `ppi=7`, 호환 `ci=1`, 전체 path depth 3
- 실제 `copyControl`·`exportControlHtml`: `sec=0`, `ppi=7`, control `0`, owner path depth 2
- 내부 클립보드 plain text `[표]`, control 포함
- HTML export에 `<table>` 포함
- Ctrl+C 뒤 객체 선택 유지
- 브라우저 warning/error 0건

## 범위 종료와 후속 이슈

작업지시자 시각 판정에서 물리 4쪽의 깊이 3 자식 표를 복사해 새 문서에 붙여넣으면 회색 헤더
셀의 배경 등 셀 스타일이 손실되는 별도 결함을 확인했다. 같은 문서 안에서는 native 내부
클립보드로 스타일이 유지되지만, 문서 교체 뒤에는 시스템 클립보드의 HTML 경로로 전환되면서
cellzone 유효 BorderFill 누락, 셀 문단 ParaShape 기본값 덮어쓰기와 글꼴 fallback이 발생한다.

이 결함은 선택·복사 주소를 바로잡는 #4272와 원인·수정 경계가 다르다. 동일 증상의 열린 이슈가
없음을 선행 검색하고 작업지시자 결정에 따라
[#4275](https://github.com/edwardkim/rhwp/issues/4275)로 분리했다. #4272는 중첩 셀 텍스트
선택·복사·붙여넣기와 중첩 표 객체 복사 경로까지 구현·검증 완료한 범위로 종료한다.

## 다음 승인 게이트

#4272의 code candidate `7da15f346`은 원격 source branch에 push했고
[PR #4276](https://github.com/edwardkim/rhwp/pull/4276)을 `devel` 대상 Open PR로 생성했다.
작업지시자 승인에 따라 maintainer self-review 기록과 대표 시각 asset을 같은 PR의 review-only 후속
commit으로 반영한다. 현재 정정은 아직 `devel`에 통합되지 않았으므로 #4272 GitHub 이슈는 닫지 않는다.
최신 head의 CI와 mergeability 확인, self-review `COMMENT` 게시와 merge는 별도 승인 게이트로 유지하고,
merge 및 `devel` 포함 확인 뒤 승인받아 이슈를 close한다.
