# PR #293 처리 결과 보고서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#293](https://github.com/edwardkim/rhwp/pull/293) |
| 작성자 | [@nameofSEOKWONHONG](https://github.com/nameofSEOKWONHONG) (SEOKWON HONG) |
| 이슈 | [#237](https://github.com/edwardkim/rhwp/issues/237) (이미 closed at 2026-04-22) |
| 처리 | **Merge (admin)** — 메인테이너 cherry-pick + A1 보강 후 |
| 처리일 | 2026-04-26 |
| Merge commit | `da55552` |

## 변경 요약

CLI 에 `export-text` / `export-markdown` 명령 추가. HWP 문서를 페이지 단위 텍스트(.txt) / 마크다운(.md) 으로 추출.

### 작성자 핵심 (`ff6d71b`)

- `src/main.rs` (+391): CLI orchestration
- `src/document_core/queries/rendering.rs` (+298): `extract_page_text_native`, `extract_page_markdown_with_images_native`, `extract_page_markdown_native`
- `src/document_core/commands/clipboard.rs` (+30): BinData 폴백
- `mydocs/plans/task_m100_237.md`, `mydocs/working/task_m100_237_stage1.md`, `mydocs/report/task_m100_237_report.md`

설계 강점:
- 계층 분리: CLI 는 I/O orchestration, DocumentCore 는 순수 데이터 추출
- 이미지 2단계 폴백 (컨트롤 참조 → BinData ID)
- 기존 API 재사용 (`detect_clipboard_image_mime`, `build_page_tree`, `paginate()`)

### 메인테이너 보강

- **A1**: 최종 보고서 내부 링크 `feature_*.md` → `task_m100_237*.md` 정정 (commit `dc45587`)
- **A2 (불필요)**: 작성자가 PR 본문에 "page_count == 0 가드 미구현" 이라 자가 보고했으나 점검 결과 `src/main.rs:545, 665` 에 이미 early return 있어 underflow 없음

## 처리 흐름 (PR #334/#335 와 동일 패턴)

1. `local/task237` 브랜치 (origin/devel 기준) cherry-pick (author=AzureAD\홍석원 보존)
2. 메인테이너 A1 보강 commit (Co-Authored-By: SEOKWON HONG)
3. 자동 검증
4. 작성자 fork (`nameofSEOKWONHONG/rhwp`) 의 `feature/export-text-markdown` 브랜치 force-push (maintainerCanModify=true)
5. CI workflow approval (first-time contributor 정책 우회)
6. CI 통과 → admin merge

## 메인테이너 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release v0.7.6` | ✅ 28.93s |
| `cargo test --lib` | ✅ 1008 passed |
| `cargo clippy --lib -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |
| `export-text samples/biz_plan.hwp` | ✅ 6 페이지 → 6 txt 파일 |
| `export-markdown samples/biz_plan.hwp` | ✅ 6 페이지 → 6 md 파일 |

## 처리 경위

1. **2026-04-22**: 작성자 첫 PR ([#237](https://github.com/edwardkim/rhwp/pull/237)) 이 base=main 으로 잘못 제출 → 작성자 본인 인지 후 close + 재제출 안내
2. **2026-04-24**: 본 PR ([#293](https://github.com/edwardkim/rhwp/pull/293)) 재제출. 메인테이너가 review 코멘트 (A1/A2/(B) 항목들) 게시
3. **2026-04-25 ~ 26**: 작성자 응답 없음 (3d+)
4. **2026-04-26**: 작업지시자 결정 — "메인테이너 측에서 마무리해서 병합 처리". 체리픽 방식으로 진행

## 후속 (별도 사이클)

review 코멘트 (B) 항목들은 후속 작업 가능:
- 구현계획서 (`task_m100_237_impl.md`)
- export-text/markdown 자동화 테스트
- MIME 확장자 매핑 확장 (tif, svg 등)

## 외부 기여 가치

- 신규 CLI 명령 2개 추가 — AI 파이프라인 / 블로그·위키 통합에 유용
- 깔끔한 계층 분리 (CLI vs DocumentCore native API)
- 알려진 이슈 정직 공개 (작성자 본인 보고)

## 참고 링크

- [PR #293](https://github.com/edwardkim/rhwp/pull/293)
- [메인테이너 첫 review 코멘트](https://github.com/edwardkim/rhwp/pull/293#issuecomment-4310954594)
- [감사 + 머지 코멘트](https://github.com/edwardkim/rhwp/pull/293#issuecomment-4322022271)
- 이슈: [#237 (closed)](https://github.com/edwardkim/rhwp/issues/237)
- 첫 PR (close): [#237](https://github.com/edwardkim/rhwp/pull/237)
