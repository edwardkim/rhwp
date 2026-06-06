# PR #1311 리뷰 — HWPX slash/backSlash type 보존

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1311 |
| 제목 | Task #1278: HWPX slash/backSlash type 보존 |
| 작성자 | Mireutale |
| 관련 이슈 | #1278 |
| 대상 브랜치 | `devel` |
| PR head | `fix/issue-1278-hwpx-diagonal-shape` / `2144ae5b` |
| 검토 기준 | `local/pr1311-upstream` |

## 2. 변경 범위

변경 파일:

| 파일 | 변경 내용 |
|---|---|
| `src/serializer/hwpx/header.rs` | `BorderFill.attr`의 slash/backSlash 방향 비트를 HWPX `<hh:slash type>` / `<hh:backSlash type>`로 복원 |

PR diff는 1파일, 54 lines 추가, 4 lines 삭제로 매우 좁다.

## 3. 문제 구조

이슈 #1278의 핵심은 HWPX parse → HWPX serialize 과정에서 대각선 방향 정보가 손실되는 문제다.

현재 parser는 HWPX borderFill의 대각선 방향 type을 `BorderFill.attr`에 저장한다.

| HWPX 요소 | 저장 위치 | 의미 |
|---|---|---|
| `<hh:slash type="...">` | `attr` bits 2..4 | slash 방향/형태 |
| `<hh:backSlash type="...">` | `attr` bits 5..7 | backSlash 방향/형태 |
| `<hh:diagonal ...>` | `bf.diagonal` | 대각선 선 종류/굵기/색 |

하지만 serializer는 기존에 `hh:slash`와 `hh:backSlash`를 항상 `type="NONE"`으로 출력했다. 따라서 HWPX를 열었다가 다시 HWPX로 저장하면 slash/backSlash 방향 정보가 사라진다.

공식 HWP5 스펙의 `BORDER_FILL` attr도 동일하게 bits 2..4와 bits 5..7을 slash/backSlash 대각선 모양 비트로 설명한다.

## 4. PR 변경 평가

PR 방향은 타당하다.

- parser가 이미 `NONE`, `CENTER`, `CENTER_BELOW`, `CENTER_ABOVE`, 그 외 `ALL`을 3비트 코드로 저장하고 있다.
- PR은 serializer에서 같은 3비트 코드를 역변환해 HWPX type 문자열로 되돌린다.
- `hh:diagonal`은 선 종류/굵기/색을 담당하고, `hh:slash`/`hh:backSlash`는 방향/형태를 담당한다는 기존 parser 계약과 맞다.
- 추가 테스트가 serializer 단위에서 `slash=CENTER`, `backSlash=CENTER_BELOW` 보존을 직접 확인한다.

주의할 점:

- `Crooked`와 `isCounter`는 이번 PR에서도 기존처럼 `0`으로 출력한다. 현재 이슈 범위는 `type` 손실 보정이므로 blocker는 아니다.
- PR base가 현재 `origin/devel`보다 이전 시점이다. `git merge-tree` 확인 결과 충돌은 없지만, 통합은 현재 `devel` 위에서 다시 적용해야 한다.

## 5. GitHub 체크

`gh pr checks 1311 --repo edwardkim/rhwp` 확인 결과:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| CodeQL | pass |
| WASM Build | skipping |

## 6. HWPX 저장 API 공개 검토

현재 코드 상태:

| 계층 | 상태 |
|---|---|
| Rust core | `serializer::serialize_hwpx(doc)` 공개 |
| DocumentCore | `export_hwpx_native()` 존재 |
| WASM | `HwpDocument.exportHwpx()` 존재 |
| rhwp-studio bridge | `wasm.exportHwpx()` 존재 |
| 일반 저장 UI | HWPX 출처 직접 저장은 `unsupported`로 차단 |
| hwpctl `SaveAs` | HWPX 출처에서 `format !== "hwp"`이면 차단 |

따라서 HWPX 저장 기능은 내부적으로 이미 상당 부분 열려 있다. 다만 사용자-facing 저장 경로에서는 #196/#888 정책에 따라 HWPX 직접 저장을 막고, HWP 변환 저장만 안내하고 있다.

이번 PR #1311은 HWPX serializer의 `borderFill` 대각선 방향 손실을 보정한다. 이는 HWPX 저장 안정성을 높이는 변경이지만, 이 한 건만으로 일반 사용자용 HWPX 직접 저장을 전면 활성화하기에는 범위가 좁다.

권장 판단:

1. **일반 저장 UI 즉시 개방은 보류**
   - `file:save`, `file:save-as`에서 HWPX 원본 덮어쓰기까지 허용하는 것은 사용자 데이터 손실 리스크가 크다.
   - README/extension 문서도 아직 HWPX 직접 저장을 베타/제한으로 안내하고 있다.

2. **명시적 API 공개는 검토 가능**
   - 이미 `exportHwpx()` 경로가 있으므로, 이를 “실험적 HWPX export API”로 문서화하거나 hwpctl 자동화 경로에서 명시적 `SaveAs(filename, "hwpx")`만 허용하는 방식은 가능하다.
   - 단, API 이름/문서/경고에는 “experimental”, “roundtrip fidelity not guaranteed”, “원본 덮어쓰기 금지 권장”을 명시해야 한다.

3. **PR #1311 통합과의 관계**
   - #1311 자체는 contributor의 좁은 serializer 수정으로 수용한다.
   - HWPX 저장 API 공개는 maintainer 후속 커밋 또는 별도 이슈/PR로 분리하는 것이 좋다.
   - 단, #1311 통합 보고서에는 “HWPX 저장 API 공개 후보 조건을 한 단계 충족했다”는 의미를 기록한다.

## 7. 권장 처리

권장안: **수용**.

이 PR은 이슈 #1278의 저장 손실 원인을 정확히 건드리고 있으며, parser/serializer의 HWPX diagonal 방향 계약을 맞추는 변경이다. 변경 폭도 작고 GitHub 체크도 통과했다.

권장 절차:

1. 현재 `devel` 기준 통합 브랜치 생성
2. PR #1311 변경 적용
3. `cargo fmt --all -- --check`
4. `cargo test --lib serializer::hwpx::header -- --nocapture`
5. `cargo test --lib parser::hwpx::header -- --nocapture`
6. 필요 시 HWPX roundtrip 샘플로 `header.xml`의 `hh:slash` / `hh:backSlash` type 보존 확인
7. 완료 보고서 작성 후 승인 게이트

## 8. PR 코멘트 초안

```markdown
검토했습니다. HWPX parser는 `hh:slash`와 `hh:backSlash`의 `type` 값을 `BorderFill.attr` bits 2..4 / 5..7에 보존하고 있는데, serializer가 이를 항상 `NONE`으로 출력하면서 HWPX roundtrip 시 대각선 방향 정보가 손실되는 구조였습니다.

이번 PR은 serializer에서 해당 3비트 값을 다시 HWPX type 문자열로 복원하므로 parser/serializer 계약과 맞습니다. `hh:diagonal`은 선 종류/굵기/색을 담당하고, `slash/backSlash type`은 방향/형태를 담당한다는 기존 구현 방향과도 일치합니다.

GitHub checks도 확인했습니다.

- Build & Test: pass
- CodeQL: pass
- Analyze jobs: pass

현재 `devel` 위에 재적용한 뒤 serializer/parser 관련 테스트를 실행하고 반영하겠습니다. 감사합니다.
```
