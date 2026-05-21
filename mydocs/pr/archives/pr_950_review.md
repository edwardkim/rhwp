# PR #950 검토 — HWPX fragment paste support (수정요청 전제)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #950 |
| 제목 | feat: add HWPX fragment paste support |
| 작성자 | dragonnite1221-lgtm (Lee doyun) — 메모리 룰 등재 누적 컨트리뷰터 |
| base ← head | `devel` ← `dragonnite1221-lgtm:public/hwpx-fragment-paste` |
| 라벨 | enhancement |
| 변경 | 21 파일 +3936 / -8 (대규모 신규 기능) |
| 연결 이슈 | 없음 |
| mergeable | MERGEABLE / **BEHIND** (rebase 필요) |
| CI | Build & Test ✅ / Analyze rust·js·py ✅ / Canvas visual diff ✅ / CodeQL ✅ / WASM skip |
| 생성 | 2026-05-17 |
| 작업지시자 코멘트 | 2026-05-18 "양식 부품 용어 기계적, 프론트엔드 의존성 추가 거버넌스 반함, 좀 더 살펴봄" |
| 이력 | **PR #880 머지 → 약 1시간 후 revert → PR #950 follow-up** |

## 2. 배경 — PR #880 머지/revert 이력

```
PR #880 머지 (6c42ee9b, 5/15 08:04)
   ↓ 약 1시간 후
Revert 1579dc8f (5/15 09:29, "Revert feat: HWPX fragment paste support")
   ↓ 이틀 후
PR #950 재제출 (5/17, 같은 컨트리뷰터, 같은 기능, follow-up)
   ↓ 다음 날
작업지시자 코멘트 (5/18): 용어 + 거버넌스 우려, 보류
```

머지 1시간 만에 revert 된 것은 매우 이례적 — 즉시 검출된 본질 우려가 있었음을 시사. PR #880 close 시 작업지시자가 정리한 **6 미해결 항목** 이 PR #950 의 출발선.

## 3. PR #880 잔존 항목 — PR #950 처리 현황 점검

| # | 항목 (PR #880 작업지시자 코멘트) | PR #950 본문 주장 | 본 검토 평가 |
|---|------|------|------|
| 1 | Dual implementation 명료화 (paste boundary 2개) | "두 boundary 는 서로 다른 safety contract — raw XML primitive vs editor-facing bridge" 본문 §Architecture note 설명 추가 | ✅ 본문 설명은 명료. 다만 코드 측 architecture comment 본 PR diff 에서 추가 검증 필요. |
| 2 | `(this.doc as any)` 타입 캐스트 | "Removed by using generated WASM binding type for `pasteHwpxFragmentInDocument`" | ⚠️ 사실 확인 필요 — diff 에서 `as any` 잔존 여부 grep 검증 (3.4) |
| 3 | 환경 의존 테스트 | "Removed local-machine yangsik smoke tests" + "self-contained bridge tests using `saved/04-blank_hwpx_empty.hwpx`" | ⚠️ saved/04-blank_hwpx_empty.hwpx (6876 bytes binary) 가 fixture 로 추가 — 적절 |
| 4 | 주석/코드 불일치 | "Aligned blank HWPX seed comments with `saved/04-blank_hwpx_empty.hwpx`" | (확인 필요) |
| 5 | 미사용 함수 | "Removed unused `build_occupied_sets` helper" | (확인 필요) |
| **6** | **예시 yangsik fragments** | "Added bundled example fragments under `rhwp-studio/public/yangsik-fragments/`" | ✅ `basic-two-cell-table.xml` / `document-title-box.xml` / `manifest.json` 3 파일 추가 — end-to-end testable 충족 |
| (추가) | HWP5 문서 호환성 | "Gated the `양식 부품` command to HWPX documents; HWP5 documents do not expose this unsupported XML-fragment operation" | ✅ HWP5 비호환 명시 + canExecute guard 추가 — 본 PR scope 정직 |
| (추가) | 카탈로그 보안 | "Hardened against traversal / double-encoded / symlinks / writable / TOCTOU / oversized" | (PR 본문 주장, 보안 review 필요시 별도) |

→ 잔존 6 항목 중 5/6 해결 (#1, #6 확실, #2~5 코드 검증 필요). HWP5 호환 추가 우려는 정직 명시.

## 4. 작업지시자 5/18 우려 사항 점검

### 4.1 ⚠️ "양식 부품" 용어 기계적 — **부분만 해소**

- ✅ UI 표시 라벨 변경: `<span class="md-label">서식 조각</span>` (index.html) — "양식 부품" 단어 UI 에서 제거
- ❌ **코드 식별자 / 디렉토리 / 클래스명 / 함수명은 모두 `yangsik-*` 로마자 잔존**:
  - 디렉토리: `rhwp-studio/public/yangsik-fragments/`
  - 명령 ID: `insert:yangsik-parts`
  - TS 클래스: `YangsikPartsDialog`
  - 함수: `fetchYangsikFragmentManifest`, `fetchYangsikFragmentXml`
  - 파일명: `ui/yangsik-parts-dialog.ts`
- **메모리 룰 `feedback_machine_vocabulary` (기계적 어휘 회피, 자연스러운 한국어 산문)** 정면 관련:
  - 'yangsik' 자체가 기계 음역. UI 라벨이 "서식 조각" 이라면 코드도 `snippet`/`form-fragment`/`form-snippet` 등 영어 의미 일관성 또는 `seosik-jogak` 일관 표기. 현재는 UI ↔ 코드 라벨 불일치.
- **수정요청 제안**: 컨트리뷰터와 의논하여 일관 명명 (옵션 A: `form-snippet` / `fragment` 영어, 옵션 B: `seosik-jogak` 일관 음역) 선택. UI/코드 표기 일치.

### 4.2 ✅ "프론트엔드 의존성 추가 거버넌스 반함" — **해소된 것으로 보임**

본 환경 PR #950 diff 점검:
- ❌ `package.json` 변경 없음 (npm 의존성 신규 추가 없음)
- ❌ `Cargo.toml` 변경 없음 (Rust crate 신규 추가 없음)
- ❌ `vite.config.ts` 변경 없음
- ❌ `vite-plugin-yangsik-fragments.ts` 추가 없음 (PR #880 에서 추가 → revert → PR #950 에서 재제출 안 함)

bundled fragments 는 `rhwp-studio/public/yangsik-fragments/` 정적 파일 (vite plugin 의존 없이 fetch 가능). 작업지시자 우려에 대응한 변경으로 판단. 본 PR body 에 명시되지 않은 점은 아쉬우나 변경 사실 자체는 정합.

### 4.3 ⚠️ 비공개 task 참조 주석 잔존

PR 변경 코드 주석에 컨트리뷰터의 로컬/비공개 작업 환경 흔적:
- `// (RCA: task_local_yangsik_paste_composed_refresh, 2026-04-28)`
- `/// (task_local_yangsik_paste_composed_refresh, RCA: 2026-04-28)`
- `/// source/sink 로 사용 (task_local_yangsik_paste_wasm_bridge).`

→ `task_local_*` 는 외부에서 추적 불가능한 비공개 참조. 메모리 룰 `feedback_external_docs_self_censor` (외부 공개 자기검열) 정합 — 공개 PR 의 코드 주석에 비공개 task ID 부적절. **수정요청**: 비공개 task 참조 제거 또는 본 PR 번호/이슈로 대체.

## 5. 검토 항목 — 코드 측면

### 5.1 변경 규모 + 구조

| 영역 | 파일 | 추가 | 비고 |
|------|------|------|------|
| Rust core | `fragment_paste.rs` | 1389 | 핵심 raw XML primitive |
| Rust core | `cross_document_migrate.rs` | 1011 | IR migration primitive (future cross-document copy/paste) |
| Rust core | `fragment_paste_in_document.rs` | 394 | editor-facing bridge (raw 재사용 + 재파싱) |
| Rust core | `wasm_api.rs` | +202 | WASM 바인딩 |
| Rust core | `queries/raw_xml.rs` | 174 | (신규 query 모듈) |
| Rust other | `document.rs`/`mod.rs`/`event.rs`/`hwpx/mod.rs` | +83/-3 | |
| Frontend | `yangsik-parts-dialog.ts` | 331 | UI 다이얼로그 |
| Frontend | `insert.ts`/`wasm-bridge.ts` | 82 | 커맨드 + 브릿지 |
| Frontend | bundled fragments + manifest | 23 | 3 파일 |
| Tests | `fragment_paste_*.rs` | 231 | integration tests |
| Fixture | `saved/04-blank_hwpx_empty.hwpx` | binary | seed |

### 5.2 ⚠️ scope 광범위 — 단일 PR 묶음 적절성

- **3 신규 Rust 모듈** (fragment_paste / cross_document_migrate / fragment_paste_in_document) 동시 도입
- `cross_document_migrate.rs` (1011 줄) 는 "future structured cross-document copy/paste" 용도 — **본 PR 의 실제 paste 기능은 byte-preserving raw XML path 만 사용**
- 본문 §Architecture note 에서 "convergence point is the command/API layer: raw HWPX fragments use the byte-preserving path now; structured IR copy/paste can use the migration primitive when that workflow is exposed" 명시
- **즉 `cross_document_migrate.rs` 는 본 PR 에서 미사용** — future use 만 위한 1011 줄 추가
- 메모리 룰 `feedback_small_batch_release_strategy` (작은 단위 PATCH 회전, 큰 묶음 위험 누적) 정면 관련. v0.x 단계에서 1011 줄 미사용 모듈 동시 도입은 과도.
- **수정요청 제안**: `cross_document_migrate.rs` 를 본 PR 에서 제거하고 future workflow 노출 시 별도 PR 분리. 본 PR scope = "raw HWPX fragment paste only".

### 5.3 PR #880 잔존 항목 4 (주석/코드 불일치) / 5 (미사용 함수) 검증 필요

- `build_occupied_sets` 함수 제거 주장 → diff 에서 잔존 여부 grep 검증 권장
- 주석/코드 정합 → 산발적이라 sampling 검증

## 6. 처리 절차 (간소화 4단계 — 수정요청 분기)

1. ✅ PR 정보 확인 (본 문서 §1~2)
2. → 본 검토 문서 작성 + 작업지시자 승인 요청 (현 단계)
3. (수정요청 절차) — 본 검토 문서를 정리해 컨트리뷰터에게 수정요청 코멘트 (작업지시자 승인 후)
4. 수정 반영 또는 close 결정 → `pr_950_report.md`

## 7. 수정요청 항목 (작업지시자 승인 후 컨트리뷰터에 전달)

### 필수 (merge blocker)

**M1. 비공개 task 참조 주석 제거** (§4.3) — 외부 공개 PR 의 코드 주석에 `task_local_*` 비공개 참조는 부적절. 본 PR 번호/이슈 또는 일반 설명으로 대체.

**M2. `cross_document_migrate.rs` 분리** (§5.2) — 본 PR 에서 미사용 1011 줄 future workflow 모듈은 별도 PR 로 분리. 본 PR scope = raw HWPX fragment paste only.

### 권고 (논의 가능)

**R1. `yangsik` 명명 일관화** (§4.1) — UI 라벨 "서식 조각" 과 코드 식별자 `yangsik-*` 불일치 해소. 옵션 A (영어 `form-snippet` / `fragment`) 또는 옵션 B (음역 `seosik-jogak`) 일관. 작업지시자 5/18 우려의 핵심.

### 사실 확인 (코드 sampling 검증)

**V1. `as any` 잔존 여부** (PR #880 잔존 #2) — `wasm-bridge.ts` 등 grep.
**V2. `build_occupied_sets` 함수 제거 확인** (PR #880 잔존 #5).
**V3. 주석/코드 정합 sampling** (PR #880 잔존 #4).

## 8. 1차 판단 (작업지시자 승인 전 잠정)

| 영역 | 평가 |
|------|------|
| PR #880 잔존 6 항목 | 5/6 본문 주장 해결 (1 항목 확인 필요 — V1/V3) |
| HWP5 호환 추가 우려 | ✅ canExecute guard + scope 정직 명시 |
| 거버넌스 우려 (의존성) | ✅ 신규 의존성 없음 (vite plugin 도 제거) |
| 용어 (`yangsik`) | ⚠️ UI ↔ 코드 불일치 — 핵심 권고 |
| 비공개 task 참조 | ⚠️ 외부 공개 부적절 — 제거 필요 |
| scope (cross_document_migrate.rs) | ⚠️ 미사용 1011 줄 — 분리 권고 |
| CI / 결정적 검증 | ✅ 통과 |
| code quality (정량) | (코드 깊이 review 미수행 — 본 검토는 scope/거버넌스/잔존 항목 중심) |

**잠정 결론**: PR 본문이 PR #880 잔존 항목을 성실히 해소했고 HWP5 호환·거버넌스 우려도 일부 대응. 그러나 (1) `yangsik` 명명 일관성, (2) 비공개 task 참조 잔존, (3) `cross_document_migrate.rs` scope 초과의 **3건 수정요청** 후 재검토 권장. 본 PR 의 핵심 기능 (raw HWPX fragment paste) 자체는 가치 있고 PR #880 검토에서 작업지시자가 "코드 품질 좋음, 인프라 잘 설계됨" 평가한 점도 정합.

**close 권고는 아님** — 수정요청 후 컨트리뷰터 응답 대기. 이전 PR #880 처럼 응답 없으면 close (작업지시자 결정).

> 본 문서는 검토 계획 + 항목 통합. 작업지시자 승인/피드백 후
> 수정요청 코멘트 작성 → 컨트리뷰터 응답 대기 → 재검토 사이클.
> 코드 깊이 review (보안, 알고리즘) 는 수정요청 통합 후 v2 검토에서.
