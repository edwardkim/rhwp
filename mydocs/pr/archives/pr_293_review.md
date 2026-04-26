# PR #293 검토 — Task #237: export text, markdown 구현 (신규 기여자)

## PR 정보

- **PR**: [#293](https://github.com/edwardkim/rhwp/pull/293)
- **이슈**: [#237](https://github.com/edwardkim/rhwp/issues/237) (CLOSED 상태로 이미 등록됨)
- **작성자**: **@nameofSEOKWONHONG (SEOKWON HONG)** — **이 저장소 첫 기여자**
- **base/head**: `devel` ← `feature/export-text-markdown`
- **Mergeable**: UNKNOWN → 확인 필요 (PR #292 머지 후 재계산)
- **CI**: ✅ 전부 SUCCESS (workflow 승인 후 실행)
- **검토일**: 2026-04-24

## 변경 요약

HWP 문서를 페이지 단위로 **텍스트(.txt)** / **마크다운(.md)** 로 추출하는 CLI 명령 2종 추가. Markdown에서는 이미지를 `{stem}_assets/` 폴더에 저장하고 링크로 치환.

### 핵심 변경 (코드 3개 파일)

| 파일 | 변경 | 설명 |
|------|------|------|
| `src/main.rs` | +405 -7 | CLI 서브커맨드 `export-text`, `export-markdown` + 옵션 파싱 + I/O orchestration |
| `src/document_core/queries/rendering.rs` | +300 | `extract_page_text_native`, `extract_page_markdown_with_images_native`, `extract_page_markdown_native` |
| `src/document_core/commands/clipboard.rs` | +30 | `get_bin_data_image_data_native`, `get_bin_data_image_mime_native` (BinData 폴백) |

### 문서
| 파일 | 포함 |
|------|------|
| `mydocs/plans/task_m100_237.md` | ✅ 수행계획서 |
| `mydocs/working/task_m100_237_stage1.md` | ✅ 단계1 보고서 |
| `mydocs/report/task_m100_237_report.md` | ✅ 최종 보고서 (링크 깨짐 ⚠️) |

## 신규 기여자 절차 준수 점검

| 규칙 | 준수 | 비고 |
|------|------|------|
| 이슈 → 브랜치 → 계획서 → 구현 순서 | △ | 수행/최종 보고서는 있음. 구현계획서(`_impl.md`) **누락** |
| 수행계획서 (`task_m100_{번호}.md`) | ✅ | 구조 타당 (배경/목표/범위/산출물/검증/리스크) |
| 구현계획서 (`task_m100_{번호}_impl.md`) | ❌ | **누락** |
| 단계별 보고서 | △ | stage1 만 존재 (범위가 작다면 허용 가능) |
| 최종 보고서 | ✅ | 존재하나 **내부 링크 깨짐** (`feature_*.md` 경로 사용, 실제 파일명과 불일치) |
| orders 갱신 | ❌ | `mydocs/orders/20260424.md` 에 Task #237 섹션 없음 |
| 브랜치명 `local/task{번호}` | ❌ | `feature/export-text-markdown` 사용 (외부 기여자로서 관대하게 볼 수 있음) |
| 커밋 메시지 `Task #N:` | ✅ | "Task #237: export text, markdown 구현" |
| Co-Authored-By AI 표기 | - | 단독 작성 (문제 없음) |
| 단일 커밋으로 압축 | ✅ | 1 커밋으로 정리됨 |
| PR 템플릿 사용 | ✅ | 변경요약/관련이슈/테스트/스크린샷 섹션 |
| Test plan 체크리스트 | △ | `cargo test`, `cargo clippy` ✅ / **SVG·WASM 확인 미체크** |
| 이슈 #237 관리 | ⚠️ | PR 템플릿과 동일한 내용으로 **이슈 본문이 부적절**. 이미 CLOSED 상태 (PR 생성 전?) |

**종합**: 외부 기여자로서 **전반적 절차 의식은 양호**(계획서/보고서 작성, PR 템플릿 사용, 커밋 스타일 준수). 일부 세부(구현계획서/orders/보고서 링크)는 후속 개선 필요.

## 기능 검증 (메인테이너 실측)

### CLI 동작 확인

```bash
# export-text
$ rhwp export-text samples/basic/KTX.hwp -o /tmp/pr293_test/
문서 로드 완료: samples/basic/KTX.hwp (1페이지)
  → /tmp/pr293_test/KTX.txt
텍스트 내보내기 완료: 1개 TXT 파일

# export-markdown
$ rhwp export-markdown samples/basic/KTX.hwp -o /tmp/pr293_md/
Markdown 내보내기 완료: 1개 MD 파일
```

- ✅ CLI `--help` 에 두 명령 등록됨
- ✅ 텍스트 / 마크다운 파일 생성
- ✅ 출력 내용에 실제 문서 텍스트 (광주/목포/경부선/호남선 등) 포함

### 빌드/테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 983 passed / 0 failed / 1 ignored |
| `cargo test --test svg_snapshot` | ✅ 6 passed |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |

### CI (GitHub Actions)

| Check | 결과 |
|-------|------|
| CI / Build & Test | ✅ SUCCESS |
| CodeQL (rust / js-ts / python) | ✅ 전부 SUCCESS |
| WASM Build | SKIPPED |

## 코드 품질 평가

### 장점
1. **계층 분리 명확** — CLI는 I/O orchestration만, 쿼리 계층(`DocumentCore::extract_*_native`)은 순수 데이터 추출
2. **이미지 2단계 폴백** — 컨트롤 참조 우선, 실패 시 BinData ID 폴백
3. **기존 API 재사용** — `detect_clipboard_image_mime`, `build_page_tree`, `paginate()` 등 기존 자원 활용
4. **의도 주석 잘 달림** — 함수 docstring으로 `TextLine 순회`, `표는 내부 순회 생략` 등 설계 의도 명시
5. **wasm32 영향 없음** — CLI 전용으로 WASM 경로 건드리지 않음

### 주의 사항
1. **알려진 이슈 기록**:
   - `page_count == 0` 가드 미구현 (작성자 기록)
   - MIME 확장자 매핑 미비 (`.bin` 폴백)
   - 후속 패치 약속됨
2. **테스트 부재**: export-text/markdown 전용 자동화 테스트 없음. 수동 CLI 동작만 확인
3. **KTX.hwp 텍스트 출력이 순서 불안정**: SVG상 2단 구조인데 텍스트는 단 경계 없이 섞임 (예상 동작인지 불명확)

### 평가

신규 기여자 첫 PR 치고는 **매우 훌륭한 품질**:
- 구조적 설계, 기존 패턴 재사용, 알려진 이슈 정직 공개
- CI 통과 + 빌드/테스트 clean

## 리스크 평가

| 리스크 | 판정 |
|--------|------|
| 기존 기능 회귀 | ✅ 전체 테스트 983/0, clippy clean |
| wasm32 호환 | ✅ CLI 전용 코드, `#[cfg(not(target_arch="wasm32"))]` 가드 적절히 사용됨 확인 필요 |
| `page_count == 0` 패닉 | ⚠️ 작성자 인지. 빈 문서 열면 underflow 가능. 후속 패치 필요 |
| 이미지 저장 실패 시 중단 | ✅ 경고 출력 후 계속 진행 (설계 의도 명시) |
| 신규 CLI 명령 충돌 | ✅ `export-svg` 와 명명 규칙 일관. `--output/-o`, `--page/-p` 옵션 체계 동일 |

## 판정

✅ **Merge 권장 (후속 개선 요청)**

**사유:**
1. **기능 자체는 완결** — 두 CLI 명령 정상 동작, 실제 문서로 검증
2. **코드 품질 양호** — 계층 분리, 기존 패턴 재사용, 명확한 주석
3. **CI + 로컬 검증 모두 통과**
4. **신규 기여자로서 절차 의식 전반 양호** — 계획서/보고서 작성, PR 템플릿 활용

**후속 요청 (merge 후 별도 커밋):**
1. 구현계획서 `mydocs/plans/task_m100_237_impl.md` 작성
2. `mydocs/report/task_m100_237_report.md` 의 내부 링크 수정 (`feature_*.md` → `task_m100_237*.md`)
3. `mydocs/orders/20260424.md` 에 Task #237 섹션 추가
4. **Known Issue 패치**: `page_count == 0` 가드, MIME 확장자 매핑 보강, export-text/markdown 자동화 테스트
5. 이슈 #237 본문 정리 (PR 템플릿과 분리)

**환영 메시지 포함한 코멘트로 피드백 전달 권장.**
