# PR #1254 처리 보고서 - 사각형 글상자 안 picture 선택/속성/삽입 지원

- **작성일**: 2026-06-03
- **PR**: #1254
- **연결 이슈**: #1171
- **브랜치**: `local/pr1254-verify`
- **병합 커밋**: `47cd32dd`
- **PR head**: `cf5fe54b33e15fd39313a0bc47e75fa83dedf862`
- **대상 샘플**: `samples/tac-img-02.hwp`

## 1. 처리 요약

PR #1254를 현재 `local/devel` (`3bd66137`) 기준 검증 브랜치에 병합했다.

병합 충돌은 없었으며, PR의 핵심 변경은 다음이다.

- 글상자 내부 picture에 `cellPath` sentinel (`cell_index=0`) 부여
- 글상자 내부 picture by-path 속성 조회/변경 지원
- 글상자 클릭 시 내부 picture 우선 hit-test
- 글상자 위 이미지 드롭 시 한컴처럼 본문 floating sibling으로 삽입
- Rust 테스트와 rhwp-studio E2E 추가

## 2. 자동 검증 결과

| 항목 | 명령 | 결과 | 비고 |
|---|---|---|---|
| whitespace | `git diff --check local/devel..HEAD` | 통과 | 출력 없음 |
| Rust fmt | `cargo fmt --all --check` | 통과 |  |
| TypeScript | `npx tsc --noEmit` (`rhwp-studio`) | 통과 | cellPath 타입 확장 확인 |
| #1171 Rust | `cargo test --test issue_1171_textbox_picture_cellpath -- --nocapture` | 통과 | 3 passed |
| 전체 integration | `cargo test --tests` | 통과 | 전체 통과, 실패 없음 |
| WASM | `docker compose --env-file .env.docker run --rm wasm` | 통과 | `pkg/` 산출물 생성, 콘솔 오류 대응 후 재빌드 |
| Studio WASM 동기화 | `cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts rhwp-studio/public/` | 통과 | public wrapper 동기화 |
| Studio build | `npm run build` (`rhwp-studio`) | 통과 | Vite production build |
| E2E 1 | `VITE_URL=http://localhost:7701 node e2e/textbox-picture-1171.test.mjs --mode=headless` | 통과 | hit-test + 속성 round-trip |
| E2E 2 | `VITE_URL=http://localhost:7701 node e2e/textbox-picture-insert-1171.test.mjs --mode=headless` | 통과 | 글상자 위 이미지 드롭 |

E2E 보고서:

- `output/e2e/textbox-picture-1171-report.html`
- `output/e2e/textbox-picture-insert-1171-report.html`

## 3. WASM/Studio 산출물

`docker compose` 빌드 후 `pkg/` 산출물을 `rhwp-studio/public/`에 동기화했다.

확인 결과:

```text
pkg/rhwp.js == rhwp-studio/public/rhwp.js
pkg/rhwp_bg.wasm == rhwp-studio/public/rhwp_bg.wasm
pkg/rhwp.d.ts == rhwp-studio/public/rhwp.d.ts
```

Git 추적 변경:

- `rhwp-studio/public/rhwp.js`
- `rhwp-studio/public/rhwp.d.ts`

참고: `rhwp-studio/public/rhwp_bg.wasm`은 현재 Git 추적 대상이 아니다.

## 4. 메인테이너 콘솔 오류 대응

메인테이너 동작 확인 중 글상자 내부 picture는 선택되지만, 삭제/크기 조정 시 다음 오류가 발생했다.

```text
렌더링 오류: 지정된 Shape 컨트롤이 그림이 아닙니다
렌더링 오류: 컨트롤 인덱스 1 범위 초과
```

원인:

- PR의 초기 구현은 hit-test와 속성 대화상자 일부 경로에서는 `cellPath`를 사용했다.
- 하지만 선택 상태 저장, 리사이즈 history target, Delete/Cut, 컨텍스트 메뉴 command, selection bbox 재조회 경로에서는 `cellPath`가 보존되지 않았다.
- 그 결과 글상자 내부 picture를 body-level `getPictureProperties`, `setPictureProperties`, `deletePictureControl`로 다시 해석하면서 외부 Shape 또는 잘못된 control index를 참조했다.

보완:

- `Cursor`의 선택 참조에 `cellPath`를 포함해 다중 선택/Shift-click/직접 선택에서 내부 picture 위치를 보존했다.
- `input-handler-picture`, `input-handler-keyboard`, `input-handler`, `insert` command, `ResizeObjectCommand`에서 `cellPath`가 있으면 by-path picture/shape API를 사용하도록 수정했다.
- WASM/Rust에 `deleteCellPictureControlByPath`를 추가해 글상자 내부 picture 삭제도 한컴 문서 구조에 맞게 inner paragraph 기준으로 처리한다.
- `cellPath` JSON 파서는 layout 쪽 긴 키(`controlIndex`, `cellIndex`, `cellParaIndex`)와 기존 짧은 키(`controlIdx`, `cellIdx`, `cellParaIdx`)를 모두 허용한다.

추가 검증:

- 글상자 내부 picture by-path 조회/수정/삭제 Rust 테스트 추가
- TypeScript 빌드와 rhwp-studio production build 통과
- WASM 재빌드 및 public wrapper 동기화
- 기존 #1171 E2E 2종 재통과

## 5. 메인테이너 동작/시각 판정표

대상 파일: `samples/tac-img-02.hwp`

| 항목 | 기대 동작 | 판정 | 비고 |
|---|---|---|---|
| 글상자 내부 picture 선택 | 6/7쪽 글상자 내부 그림 클릭 시 텍스트 편집이 아니라 picture 객체 선택 진입 | 통과 |  |
| picture 속성 대화상자 | 선택된 내부 picture의 속성 조회/수정 가능 | 통과 |  |
| 내부 picture hit-test | 글상자 Shape와 내부 picture가 겹칠 때 picture 우선 선택 | 통과 |  |
| 글상자 내부 picture 삭제/크기 조정 | 선택된 내부 picture 삭제/리사이즈 시 `cellPath` by-path 경로 유지 | 통과 | 콘솔 렌더링 오류 제거 |
| 글상자 위 이미지 드롭 | 글상자 내부가 아니라 본문 floating sibling으로 삽입 | 통과 | 한컴 정합 |
| 기존 표 셀 picture | 표 셀 내부 picture 선택/속성 흐름 회귀 없음 | 통과 |  |

메인테이너 판정:

```text
2026-06-03 통과
```

관찰된 콘솔 성능 경고:

```text
[Violation] 'keydown' handler took 214ms
[Violation] 'setTimeout' handler took 58ms
[Violation] 'setTimeout' handler took 58ms
[Violation] 'setTimeout' handler took 60ms
```

위 로그는 Chrome long-task 성능 경고이며, 이번 PR에서 해결한 `지정된 Shape 컨트롤이 그림이 아닙니다` / `컨트롤 인덱스 범위 초과` 렌더링 오류와는 별개다. 기능 동작은 통과로 판정하되, 대형 문서에서 삭제/리사이즈 후 재조판이 keydown/render timer를 오래 점유하는 성능 관찰 항목으로 남긴다.

## 6. 남은 절차

메인테이너 동작/시각 판정이 통과되었으므로 다음을 진행한다.

1. `local/devel`에 검증 브랜치 반영
2. 변경 문서와 WASM wrapper 커밋
3. `local/devel:devel` push
4. PR #1254 및 Issue #1171 종료 처리

## 7. 현재 결론

자동 검증과 메인테이너 동작 판정 기준으로 **수용 가능**하다.
