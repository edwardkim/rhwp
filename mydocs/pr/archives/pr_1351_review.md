# PR #1351 검토 — useFontSpace 직렬화 시 항상 false 고정 수정

- PR: https://github.com/edwardkim/rhwp/pull/1351
- 제목: Task #1350 useFontSpace 직렬화 시 항상 false 고정 (IR 값 무시)
- 작성일: 2026-06-10
- 작성자: `Martinel2`
- 관련 이슈: #1350 "HWPX 시리얼라이저에서 useFontSpace 직렬화 시 항상 false 고정 (IR 값 무시)"
- base: `devel` (`54e6393c`)
- head: `Martinel2:devel` (`232d6c14`)
- 로컬 검토 브랜치: `local/pr1351-upstream`

## 1. 요약 판단

**조건부 수용 권고**.

PR의 본질인 `useFontSpace` IR 보존과 HWPX 직렬화 수정은 타당하다. 현재 `local/devel`에는
`CharShape.use_font_space`가 없고, `src/serializer/hwpx/header.rs`는 `useFontSpace`를
항상 `0`으로 내보낸다. 따라서 #1350의 문제는 실제로 존재한다.

다만 PR 브랜치 자체는 오래된 `devel`에서 갈라진 fork의 `devel` 브랜치로 보이며, #1298/#1321
계열 과거 커밋과 문서가 함께 섞여 있다. 현재 `local/devel`과 PR head를 직접 비교하면 이미
반영된 최근 작업들이 대량 삭제/되돌림처럼 나타난다. 그러므로 **GitHub merge 또는 PR 전체
merge는 금지**하고, #1350 본질 커밋 `d9107849`만 선별 cherry-pick하는 것이 맞다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | MERGEABLE |
| 변경량 | 18 files, +1176 / -2 |
| 작성자 | `Martinel2` |
| 관련 이슈 | #1350 |

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

## 3. 커밋 구조

PR head에는 다음 계열이 함께 포함되어 있다.

- #1298 0-length field range 보정 커밋/문서
- #1321 빈 문단 0-length field 보정 커밋/문서
- fork `devel` 내부 merge commit들
- #1350 본질 커밋:
  - `d9107849` — `fix: useFontSpace IR 필드 추가 및 HWP5/HWPX 파서·직렬화기 반영 (#1350)`

수용 후보:

- `d9107849`

제외 후보:

- #1298/#1321 계열 커밋 및 문서
- PR 내부 merge commit `82f497b5`, `232d6c14`
- fork `devel` 동기화 merge commit `f1165a0c`

## 4. 변경 검토

### 4.1 본질 변경

`d9107849`는 다음 5개 파일을 변경한다.

| 파일 | 변경 |
|---|---|
| `src/model/style.rs` | `CharShape.use_font_space: bool` 추가, `PartialEq` 반영 |
| `src/parser/doc_info.rs` | HWP5 CharShape attr bit 25를 `use_font_space`로 파싱 |
| `src/parser/hwpx/header.rs` | HWPX `useFontSpace` 속성 파싱 |
| `src/serializer/hwpx/header.rs` | `useFontSpace` 하드코딩 `false` 제거, `cs.use_font_space` 출력 |
| `src/serializer/doc_info/tests.rs` | 명시적 `CharShape` 리터럴 보완 |

검토 결과:

- HWP5 `attr` bit 25와 HWPX `useFontSpace` 속성을 같은 IR 필드로 연결하는 방향은 맞다.
- HWPX serializer에서 `bool01(false)`를 `bool01(cs.use_font_space)`로 바꾸는 것은 #1350의
  직접 원인에 대한 정정이다.
- `CharShape::PartialEq`에 새 필드를 포함한 것도 roundtrip 비교 관점에서 타당하다.
- 명시적 `CharShape` 리터럴은 PR 내 테스트 2곳만 보완했지만, 현재 `local/devel`의 나머지
  리터럴은 대부분 `..Default::default()`를 사용하므로 적용 가능성은 높다.

### 4.2 적용 가능성

현재 `local/devel` 기준으로 `d9107849` patch는 적용 가능하다.

확인:

```text
git show --format= d9107849 -- src/model/style.rs src/parser/doc_info.rs src/parser/hwpx/header.rs src/serializer/doc_info/tests.rs src/serializer/hwpx/header.rs | git apply --check
```

결과: 통과

## 5. 주요 리스크

### 5.1 PR 전체 merge 위험 — 높음

PR head가 현재 `devel` 최신 상태를 기준으로 하지 않는다. 직접 merge하면 최근 PR 처리 문서,
Studio/renderer 관련 변경, 테스트 파일 등이 대량 삭제 또는 되돌림 형태로 충돌할 수 있다.

판정:

- PR 전체 merge 금지
- `d9107849` 단일 cherry-pick만 수용 후보

### 5.2 HWP5 serializer bit 25 재구성 누락 — 중간

PR은 HWP5 파서에서 bit 25를 읽지만, `src/serializer/doc_info.rs::serialize_char_shape()`는
`cs.use_font_space`를 attr bit 25에 반영하지 않는다.

영향:

- HWP 원본에서 온 `CharShape`는 `raw_data`가 우선되어 HWP5 재저장 시 기존 bit 25가 보존된다.
- 그러나 HWPX에서 파싱했거나 새로 생성한 `CharShape`처럼 `raw_data`가 없는 경우,
  `use_font_space=true`여도 HWP5 저장 시 bit 25가 빠질 수 있다.

이번 PR의 주 목표가 HWPX serializer 결함 수정이라는 점에서는 blocker로 보지 않는다. 다만
IR 필드를 추가한 이상, 수용 시 메인테이너 보강 커밋으로 HWP5 serializer bit 25 반영과 테스트를
추가하거나 후속 이슈로 등록하는 것이 좋다.

### 5.3 렌더링 시각 판정 필요성 — 낮음

현재 렌더러가 `use_font_space`를 직접 사용하는 경로는 확인되지 않았다. 이번 PR은 parsing/export
metadata 보존에 가깝다. 따라서 SVG/WASM 시각 판정보다는 serializer roundtrip 테스트가 핵심이다.

## 6. 권고 수용 절차

작업지시자 승인 후:

1. `local/devel` 기준으로 `d9107849` 단일 cherry-pick
2. 필요 시 메인테이너 보강 커밋 추가
   - `serialize_char_shape()` attr bit 25에 `cs.use_font_space` 반영
   - `use_font_space=true` HWP5 serializer 회귀 테스트 추가
3. 검증
   - `cargo fmt --check`
   - `cargo test --lib test_parse_char_shape_use_font_space -- --nocapture`
   - `cargo test --lib write_char_pr_use_font_space_roundtrip -- --nocapture`
   - `cargo test --lib test_serialize_char_shape_roundtrip -- --nocapture`
   - `cargo clippy --lib -- -D warnings`
4. 처리 보고서 작성
5. 승인 시 `devel` no-ff merge, push, PR #1351 close, issue #1350 close

## 7. 승인 요청

위 검토 결과 기준으로 PR #1351을 **`d9107849` 단일 cherry-pick + HWP5 serializer bit 25 보강
검토** 방식으로 진행해도 되는지 승인 요청한다.
