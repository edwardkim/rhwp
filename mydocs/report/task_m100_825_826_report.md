# Task #825 + #826 최종 결과 보고서

**제목**: sample11.hwp 머리말 영역 결함 2건 통합 정정 (+ Task #824 통합 머지)
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task825_826` (base: `local/devel`)
**이슈**:
- [#824](https://github.com/edwardkim/rhwp/issues/824) — HWP3 임베디드 그림 external_path 오표시 (Task #741 후속)
- [#825](https://github.com/edwardkim/rhwp/issues/825) — rhwp-studio 머리말 그림 우클릭 → 개체 속성 dialog 미표시
- [#826](https://github.com/edwardkim/rhwp/issues/826) — HWP3 PUA U+F080F / U+F0827 글리프 부재
- 후속: [#831](https://github.com/edwardkim/rhwp/issues/831) — picture 회전/대칭 미지원 (별도 등록)

## 결론

3 결함 통합 정정 완료. 작업지시자 시각 판정 통과. 통합 PR 로 `closes #824 + #825 + #826`.

## 결함별 정정 내용

### #824 — HWP3 임베디드 그림 external_path 오표시
**본질**: `src/parser/hwp3/mod.rs:935-952` 의 `pic_type 0/1/2` 동일 처리 → 임베디드 그림 (pic_type==2) 도 external_path 설정.
**정정**: `if pic_type == 0` 가드 1줄 추가. `bin_data_id` 매핑은 type 무관 유지.
**원본 task**: PR #827 (별도 close, 본 통합 PR 에서 흡수).

### #825 — 머리말/꼬리말 picture 선택 + dialog
**본질** (다층):
1. `layout_picture` 가 머리말 호출 시 indices None 전달 → ImageNode 메타 부재 → findPictureAtClick 필터링
2. TAC 인라인 picture (sample11 머리말 ￼) 는 layout_paragraph 경로로 처리되어 동일 결함
3. `getPictureProperties` 가 본문 lookup 만 → 머리말 picture 도달 불가
4. cursor.selectedPictureRef 에 headerFooter 정보 부재
5. 머리말 편집 모드 click 처리가 picture hit-test 우선순위 미지원

**정정** (Stage 3a~3d):
| Stage | 변경 |
|---|---|
| 3a | `ImageNode.header_footer_ref` 필드 + `HeaderFooterImageRef` / `HeaderFooterKind` 신규 타입. `layout_picture_full()` 신규. `layout_header_footer_paragraphs` 시그니처 확장. |
| 3a 보강 | `propagate_header_footer_ref()` 후처리 (TAC 인라인 picture 의 ImageNode 후처리, para_index 정규화 `usize::MAX-i → i`). |
| 3b | `getPageControlLayout` JSON 에 `headerFooter` 필드 출력. `get_header_footer_picture_properties_native` + WASM 바인딩. |
| 3c | findPictureAtClick 반환 타입 확장. picture-props-dialog `open()` 시그니처 확장 → 신규 API 분기. |
| 3c 보강 1 | 머리말 편집 모드 click handler 에 picture hit-test 우선 호출. |
| 3c 보강 2 | 모든 enterPictureObjectSelectionDirect 호출 (line/shape/image) 에 headerFooter 전파. |
| 3d | `set_header_footer_picture_properties_native` + WASM 바인딩. picture-props-dialog 저장 분기. `apply_picture_props_inner` helper 분리. |

### #826 — HWP3 PUA U+F080F / U+F0827 글리프 부재
**본질**: HWP3 char `0x301C` / `0x303D` → 한컴 PUA `U+F080F` / `U+F0827` 매핑 (HWP5 cross-ref 정합 의도). 한컴 함초롬 폰트는 PUA glyph 보유, rhwp-studio 번들 폰트 (오픈 라이선스) 부재.
**정정**: `paragraph_layout.rs:3052` 기존 0xF00D0~0xF09FF match 에 2 케이스 추가 — render-time substitution.
- `U+F080F` → `U+2501` (━ HEAVY HORIZONTAL)
- `U+F0827` → `U+25A0` (■ BLACK SQUARE, 잠정)

**측정/렌더링 양쪽 자동 적용**. IR/parser 무변경 → HWP5 cross-ref 정합 유지. PR #753 (Task #741) 본문에 명시된 후속 항목 직접 수행.

## 검증

| 항목 | 결과 |
|---|---|
| `cargo test --release` 전체 | 1360 passed, 0 failed |
| `cargo test --test issue_824` | 2 PASS |
| `cargo test --test issue_825` | 3 PASS (sanity + 회귀 가드 + GREEN 신규 API) |
| `cargo test --test issue_826` | 4 PASS (RED→GREEN 2 + 회귀 가드 2) |
| `cargo clippy --release -- -D warnings` | clean |
| `npx tsc --noEmit` | clean |
| WASM 재빌드 | 26.07s wasm-opt 완료 |
| SVG 회귀 (886 page 비교) | 회귀 0 (sample10 272 diff = 의도 PUA 변환) |
| 작업지시자 시각 판정 | ✅ 통과 |

### 시각 판정 통과 확인 (Stage 5 직전)

- ✅ sample11.hwp 머리말 가로선 패턴: `━━━` 정상 표시 (#826)
- ✅ 머리말 DCT 그림 클릭 → 선택 핸들 표시 (#825 hit-test)
- ✅ 우클릭 → "개체 속성" 메뉴 표시 (#825 context menu)
- ✅ 개체 속성 dialog 정상 표시 (#825 view)
- ✅ 파일 이름 빈 값 + 문서에 포함 체크됨 (#824 한컴오피스 2022 정합)
- ✅ 속성 변경 + [설정] 저장 작동 (#825 set 경로)

## 단계 진행 + commit 요약

| 단계 | commit | 내용 |
|---|---|---|
| 0 | (Task #824 4 commits merged) | `pic_type` 분기 + fixture |
| 1 (RED) | `?` | 사전 조사 + RED 테스트 |
| 2 (GREEN #826) | `d1152ec` | PUA substitution 2 케이스 |
| 3a+3b | `051bf2b` | Render chain + WASM API |
| 3c | `41550b3` | TypeScript UI |
| 3c 보강 1 | `36f9d86` | 머리말 모드 picture click 감지 |
| 3c 보강 2 | `4b20082` | enterPictureObjectSelectionDirect 전파 |
| 3a 보강 | `c97d971` | propagate_header_footer_ref 후처리 |
| 3d | `4a2dcdc` | set 경로 (setHeaderFooterPictureProperties) |
| Task #824 merge | (merge commit) | PR #827 통합 |
| 4 | `d927488` | 회귀 검증 + clippy 정정 |
| 5 (최종) | (본 commit) | 보고서 + orders + closes 양쪽 |

## 산출물

- `tests/issue_824.rs`, `tests/issue_825.rs`, `tests/issue_826.rs` — 회귀 테스트
- `src/parser/hwp3/mod.rs` (#824), `src/renderer/layout/paragraph_layout.rs` (#826), `src/renderer/render_tree.rs` / `layout.rs` / `picture_footnote.rs` / `table_cell_content.rs` (#825 render), `src/document_core/queries/rendering.rs` / `commands/object_ops.rs` / `wasm_api.rs` (#825 API)
- `rhwp-studio/src/engine/input-handler-mouse.ts` / `input-handler.ts` / `cursor.ts`, `core/wasm-bridge.ts`, `command/commands/insert.ts`, `ui/picture-props-dialog.ts` (#825 UI)
- Fixture: `samples/hwp3-sample11.hwp` + `-hwp5.hwp` + `-hwpx.hwpx`, `pdf/hwp3-sample11-hwpx-2022.pdf` (Task #824 merge)
- 문서: 수행/구현 계획서 + 단계 1-5 보고서 + 본 최종 보고서

## 알려진 제약 (별도 follow-up)

- **머리말/꼬리말 picture 캡션 신규 생성** — `set_header_footer_picture_properties_native` 가 NotSupported 에러 (현행 dialog UI 가 머리말 picture 캡션 변경을 노출하지 않으므로 실용 영향 없음)
- **머리말 picture 신규 삽입 / 삭제 기능** — 별도 task
- **picture (이미지) 회전/대칭 버튼 미지원** — Issue [#831](https://github.com/edwardkim/rhwp/issues/831) 별도 등록 (모든 picture 공통 결함, Task #825 scope 외)
- **U+F0827 시각 정합** — 매핑 후보 `U+25A0 ■` 잠정. 실제 함초롬 글리프 미확인. sample11 에 등장 부재로 시각 검증 미수행. 추후 등장 sample 발견 시 재조정 (johab.rs 코멘트 외 상세 부재).

## 메모리 룰 정합

- `feedback_visual_judgment_authority` ✅ — 작업지시자 시각 판정 게이트 통과 후 commit (4회 iteration: 머리말 모드 click → 본문 모드 click → TAC 후처리 → Task #824 merge)
- `feedback_pr_supersede_chain` (b) 패턴 — Task #741 (PR #753) / Task #824 (PR #827) 후속 결함 통합
- `feedback_process_must_follow` ✅ — 수행/구현 계획서 → 5단계 절차 준수
- `feedback_image_renderer_paths_separate` 권위 사례 강화 — picture render 의 다중 경로 (layout_picture / layout_paragraph TAC) 비대칭 결함 식별 + 후처리 통합 정합
