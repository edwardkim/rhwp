# PR #1175 검토 — external image reference discovery contract (Task #1142)

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1175 |
| 제목 | Task #1142: external image reference discovery contract 추가 |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) — 핵심 컨트리뷰터 |
| base ← head | `devel` ← `postmelee:local/task1142` (cross-repo) |
| 상태 | OPEN, MERGEABLE (BEHIND) |
| 변경 | +225 / -30, 2 파일 (`src/wasm_api.rs`, `tests/issue_1142.rs`) |
| 라벨 | enhancement / v1.0.0 |
| 연결 | Closes #1142 (Parent #1141, Refs #536/#741/#873/#875) |
| CI | 전부 pass (Build&Test, Analyze rust/js/py, Canvas visual diff, CodeQL) |

## 2. 컨트리뷰터 이력

@postmelee: #1174(#1144, 직전 머지), #1163(#1017), #1019(#975) 등. PageLayerTree/렌더러/이미지 영역 핵심 기여자.

## 3. 변경 내용

렌더러 consumer 가 렌더 전 외부 linked image 리소스를 준비할 수 있도록 **구조화된 discovery API** 추가. 이번 PR 은 **discovery contract 만** (bytes 주입/resolver 는 후속 #1143).

- `getExternalImageReferences()` 신규: `key` / `binDataId` / `originalPath` / `basename` / `extension` / `loaded`
- `collect_external_image_references`: `Control::Picture` + `Control::Shape(ShapeObject::Picture)` 양쪽 `external_path` 수집, `bin_data_id` 로 중복 제거
- `external_image_loaded`: **index-first** (`bin_data_content[bin_data_id-1]`) → renderer lookup 과 동일 규칙
- 기존 `getExternalImageBasenames()` 는 문자열 배열 계약 유지하며 새 discovery 로직 재사용

## 4. 코드 검토

- 순수 **opt-in API 추가** (기존 동작 무변경, 하위 호환 100%). `feedback_small_batch_release_strategy` 정합.
- `bin_data_id != BinDataContent.id` 가능성을 고려해 index-first 판정을 **테스트로 고정** (PR 본문 명시) — 견고.
- file search path / bytes loading / injection 은 의도적으로 제외 (#1143 분리) — 작은 단위 회전.

## 5. 충돌·회귀 점검

- BEHIND. 코드 충돌: **없음** — `git merge origin/devel` auto-merge clean (충돌 파일 0).
- 2 파일 모두 devel 최근 변경과 무충돌.
- clippy 비고: PR 본문에 `--all-targets -D warnings` 는 기존 테스트/모듈 warning 으로 실패하나, 변경 범위(lib + issue_1142) clippy 는 통과. (저장소 기존 부채, 본 PR 무관)

## 6. 판단 (잠정)

순수 discovery API 추가 + index-first 판정 테스트 고정 + 하위 호환 + CI pass + 충돌 없음. **merge 권장**.
- 처리: BEHIND → devel 직접 머지 (fork push 권한 없음).
- 시각 영향 없음(API 추가) → 빌드+테스트로 검증, 시각 판정 불요.

> 승인 시: auto-merge 검증(빌드+test issue_1142) → merge → `pr_1175_report.md`.

## 비고 (검토 과정)

초기 PR 번호로 주제를 추정하지 않고 실제 PR 정보 확인. (#1174 오추정 교훈 반영)

---

## 7. 처리 결과 (보고)

- **MERGED**: devel `a19d8e8d` (이슈 #1142 close)
- auto-merge clean (wasm_api.rs #1144 와 다른 영역, 충돌 0)
- 검증: 빌드 + issue_1142 5 + cargo test --tests 전수 + fmt 통과
- PR 코멘트 등록 + 이슈 #1142 close
