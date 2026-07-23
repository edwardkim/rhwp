# M100 #2308 Stage 2 — revision 및 overlay 기반 도입

## 기준

- 브랜치: `issue-2308-hyper-waterfall-rebuild`
- 기준선: `upstream/devel@cbddc1cd87084b60685da9a2b4369a4511d86173`
- 완료일: 2026-07-23

## 구현

- `RenderNormalizationState`에 document epoch, section revision, logical path revision ledger를
  추가했다.
- `RenderPathEntry`는 표 셀, 표 캡션, 도형 글상자, 그림 캡션을 명시적인 variant로 표현한다.
- API 호환용 표 캡션 sentinel은 mutation 입력 경계에서만 해석하고 derived-state key에는 저장하지
  않는다.
- `RenderNormalizationOverlay`와 `NestedTableWidthProjection`을 추가했다. 논리 경로 map이 권위
  identity이며 현재 source `Table` pointer map은 hot-path index다.
- 동일 source path와 폭에서는 기존 projection `Arc`를 재사용하고, source path가 사라지면 stale
  projection을 제거한다.
- 기존 `mark_section_dirty()`는 section revision까지 무효화하는 안전 기본값으로 유지하고,
  구조가 안정적인 path edit용 pagination-only 경계를 분리했다.

Stage 2에서는 기반 타입과 revision 계약만 도입했다. 기존 #2004/#2195
`render_normalized` 소비 및 deferred mirror는 각각 Stage 3과 Stage 4에서 이전하므로 RED guard는
아직 종료 조건이 아니다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib render_normalization::tests` | 4 passed |
| `cargo test --lib issue_2308_` | 2 passed |
| `cargo fmt --all` | PASS |

검증된 계약:

- source table/cell width 불변
- stable projection `Arc` 재사용
- unrelated path edit 뒤 sibling projection 재사용
- source path 제거 뒤 stale projection 미사용
- 네 가지 편집 경로의 명시적 revision key
- `DocumentCore: Send`
