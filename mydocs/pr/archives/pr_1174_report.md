# PR #1174 검토 — filename field context 멀티 렌더러 경로 일관화 (Task #1144)

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1174 |
| 제목 | Task #1144: filename field context를 멀티 렌더러 경로에서 일관화 |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) — 핵심 컨트리뷰터 |
| base ← head | `devel` ← `postmelee:local/task1144` (cross-repo) |
| 상태 | OPEN, MERGEABLE (BEHIND) |
| 변경 | +160 / -1, 3 파일 |
| 라벨 | enhancement / v1.0.0 |
| 연결 | Closes #1144 |
| CI | 전부 pass (Build&Test, Analyze rust/js/py, Canvas visual diff, CodeQL) |

## 2. 컨트리뷰터 이력

@postmelee: #1163(#1017 z-order, 어제 머지), #1019(#975 워터마크), #1018/#982 등. PageLayerTree replay·filename·워터마크 영역 핵심 기여자.

## 3. 근본 원인 (PR 기재 + 검증)

filename field 치환 경로(`DocumentCore.file_name → LayoutEngine.set_file_name → header/footer marker 치환 → PageRenderTree → PageLayerTree`)는 이미 존재하나:
- `setFileName` 이 `DocumentCore.file_name` 만 바꾸고 **PageRenderTree cache 를 무효화하지 않음** → cache 가 filename-resolved text 를 보관하므로 stale filename 잔존.
- rhwp-studio `fileName` setter 가 `_fileName` 만 갱신, WASM core `setFileName` 미호출 → Save/Save As 경로와 core 어긋남.

## 4. 변경 내용 (3 파일)

| 파일 | 변경 |
|------|------|
| `src/wasm_api.rs` (+3) | `set_file_name`: 값이 실제 바뀐 경우만 `invalidate_page_tree_cache()` 호출. public API signature 유지 |
| `rhwp-studio/src/core/wasm-bridge.ts` (+12) | `fileName` setter 가 `this.doc?.setFileName(name)` 도 호출 |
| `tests/issue_1144.rs` (+145, 신규) | 회귀 테스트 5개 |

## 5. 코드 검토

- `invalidate_page_tree_cache()` 는 기존 메서드(코드 8곳 사용) — 신규 위험 없음. 값 동일 시 무효화 skip (불필요 재계산 방지) — 적절.
- 치환 규칙 자체는 layout 단계 유지 (renderer backend 가 추측 안 함). #982 displayText/displayPositions contract 미변경.
- **새 샘플/LFS fixture 없이** command API(`create_blank_document` + `apply_hf_template`)로 동적 fixture 구성 → 회귀 고정. 작은 단위 회전 정책 정합.

## 6. 테스트 (5)

- PageLayerTree text 가 setFileName 값 포함
- raw marker `\u{0017}` 미잔존
- 첫 렌더 후 filename 변경 시 stale 미잔존 (cache invalidation)
- CanvasKit replay plan entrypoint 가 cache 선생성 후에도 새 filename 반영
- native-skia Skia PNG export 도 동일 cache invalidation (cfg-gated)

## 7. 충돌·회귀 점검

- BEHIND (devel 최신 미반영). 코드 충돌: **없음** — `git merge origin/devel` auto-merge clean.
- 3 파일 모두 devel 최근 변경과 무충돌.

## 8. 판단 (잠정)

작고 명확한 캐시 무효화 정정 + 회귀 5개 + 새 fixture 없음 + CI pass + 충돌 없음. **merge 권장**.
- 처리: BEHIND → devel 직접 머지 (fork push 권한 없음).
- filename field 는 머리말/꼬리말 텍스트 — Canvas visual diff CI pass + 테스트로 검증, 시각 판정은 선택.

> 승인 시: auto-merge 검증(빌드+테스트) → merge → `pr_1174_report.md`.

## 비고 (검토 과정)

초기 PR 번호만으로 "탭/리더" 로 오추정했으나, 실제 PR 정보 확인 후 정정 — filename field context 일관화가 정확한 주제.

---

## 9. 처리 결과 (보고)

- **MERGED**: devel `804b4ae9` (이슈 #1144 close)
- auto-merge clean (3 파일 무충돌), BEHIND → devel 직접 머지
- 검증: 빌드 + issue_1144 5(native-skia 포함) + cargo test --tests 전수 + fmt 통과
- PR 코멘트 등록 + 이슈 #1144 close
