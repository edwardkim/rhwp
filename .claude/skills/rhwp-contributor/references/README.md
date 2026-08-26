# rhwp-contributor 레퍼런스

SKILL.md 가 라우터다. 이 폴더의 문서는 단계별 정본 인용과 실행 레시피다.
새 CLI 없음. gym 없음. DocumentCore 편집 로직 발명 없음.

| 파일 | 담당 |
|------|------|
| [procedure-order.md](procedure-order.md) | 필수 8단 순서 |
| [issue-first.md](issue-first.md) | 이슈 선등록·중복 PR |
| [analyze-canonical.md](analyze-canonical.md) | 정본 문서·계약 시험 |
| [branch-isolation.md](branch-isolation.md) | `upstream/devel` 브랜치 |
| [isolation-worktree.md](isolation-worktree.md) | 격리 워크트리·이름 금지 |
| [implement-scope.md](implement-scope.md) | 구현 범위·금지 축 |
| [staging-named-files.md](staging-named-files.md) | `git add -A` 금지 |
| [fmt-hard-gate.md](fmt-hard-gate.md) | `cargo fmt --all -- --check` |
| [rustfmt-unix.md](rustfmt-unix.md) | `newline_style=Unix` |
| [clippy-and-tests.md](clippy-and-tests.md) | clippy · 관련 test |
| [visual-evidence.md](visual-evidence.md) | 렌더/레이아웃 근거 |
| [work-receipt-pointers.md](work-receipt-pointers.md) | replay/audit/lineage 포인터 |
| [working-doc.md](working-doc.md) | `mydocs/working/` |
| [korean-pr.md](korean-pr.md) | 한국어 PR · `--body-file` |
| [pr-template-checkboxes.md](pr-template-checkboxes.md) | 첫 체크박스 = fmt |
| [exceptions.md](exceptions.md) | 스파스·CRLF·중복·noci |
| [pitfalls.md](pitfalls.md) | 에이전트 실록 |
| [decision-tree.md](decision-tree.md) | 요청 → 단계 |
| [recipe-index.md](recipe-index.md) | 예제·픽스처 교차표 |
| [command-field-catalog.md](command-field-catalog.md) | 허용 명령·봉투 키 |

생성기: [`_gen_pack.py`](_gen_pack.py). 결정론. 바이너리 없음.
