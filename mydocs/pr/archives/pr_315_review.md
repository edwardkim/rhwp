# PR #315 검토 — Epic #309 1차: vpos 검증 도구 + TypesetEngine 발견

## PR 정보

- **PR**: [#315](https://github.com/edwardkim/rhwp/pull/315)
- **이슈**: #311 (vpos-reset 강제 분리 가설), #312 (column 정확도 조사) — 둘 다 CLOSED
- **작성자**: @planet6897
- **base/head**: `devel` ← `task312`
- **Mergeable**: ⚠️ 원래 CONFLICTING (orders 1건 충돌, 코드 충돌 없음)
- **CI**: ✅ 전부 SUCCESS

## 변경 요약

Epic #309 (페이지네이션 정확도) 의 두 sub-issue 통합 PR. **default 동작 변경 0** — 모든 신규 기능이 옵트인 또는 진단 출력.

### 1. Task #311 — vpos-reset 강제 분리 가설 부정

**가설**: FullParagraph 내부 vpos-reset 위치에서 단/페이지를 강제 분리하면 21_언어 PDF 일치

**검증 결과**: 
| 샘플 | OFF | ON |
|------|-----|-----|
| 21_언어 | 19 | **20 (+1)** ❌ |
| exam_math | 20 | 20 |
| exam_kor | 25 | 25 |
| exam_eng | 11 | 11 |

→ **가설 부정**. 우리 엔진의 column 가용 공간이 HWP보다 관대하여 분리만으로는 누적 overflow 해소 안 됨.

**산출물 (보존)**: `PaginationOpts` 구조체, `paginate_with_forced_breaks` 메서드, `--respect-vpos-reset` CLI 플래그 (기본 off).

### 2. Task #312 — column 정확도 조사 (의외의 발견)

**가설 부정**: 단일 column origin 모델로 차이를 설명 불가 (페이지마다 다른 부호/크기 diff).

**의외의 발견**: `dump-pages` 의 `TYPESET_VERIFY` 로그에서 코드베이스에 이미 존재하는 `TypesetEngine` 이 PDF 와 더 가까운 결과 산출:

| 샘플 | Paginator | TypesetEngine | PDF |
|------|-----------|---------------|-----|
| 21_언어 | 19 | **15** | 15 ✅ |
| exam_math | 20 | 20 | 20 ✅ |
| exam_kor | 25 | 24 | (미보유) |
| exam_eng | 11 | 9 | (미보유) |

**산출물 (도구)**: `ColumnContent.used_height` 필드, `dump-pages` 단 헤더 `used`/`hwp_used`/`diff` 출력.

## 변경 범위 분석

### 코드 (10파일, default 동작 변경 0)

| 파일 | 변경 | 안전성 |
|------|------|--------|
| `pagination.rs` | `PaginationOpts` 구조체 + `ColumnContent.used_height` | ✅ 기본값 동일 |
| `pagination/engine.rs` | `paginate_with_forced_breaks` 메서드 추가 | ✅ 새 메서드, 기존 호출 안 건드림 |
| `pagination/state.rs` | `flush_column` 에서 `used_height` 저장 | ✅ 데이터만 추가 |
| `typeset.rs` | `used_height` 저장 | ✅ 동일 |
| `document_core/mod.rs` `commands/document.rs` | `respect_vpos_reset` 필드 (default false) | ✅ 옵트인 |
| `queries/rendering.rs` | `dump_page_items` used/hwp_used/diff 출력 | ✅ 진단 출력 |
| `wasm_api.rs` | `set_respect_vpos_reset` 셋터 | ✅ 옵트인 |
| `main.rs` | `--respect-vpos-reset` CLI 플래그 | ✅ 옵트인 |
| `layout/tests.rs` | fixture 5건 업데이트 | ✅ 테스트만 |

### 문서 (14개)

수행계획서 + 구현계획서 + 단계 보고서 + 최종 보고서 + tech 분석 부록 — 두 task 모두 완비.

## 메인테이너 검증 결과

### PR 브랜치 + devel merge 후

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **992 passed / 0 failed / 1 ignored** |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 유지) |
| `cargo test --test issue_301` | ✅ 1 passed (#301 회귀 없음) |
| `cargo test --test tab_cross_run` | ✅ 1 passed (#290 회귀 없음) |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |

### 신규 진단 도구 동작 검증

```bash
$ rhwp dump-pages "samples/21_언어_기출_편집가능본.hwp" -p 2
=== 페이지 3 (global_idx=2, section=0, page_num=3) ===
  단 0 (items=7, used=1219.5px, hwp_used≈1209.9px, diff=+9.5px)
  단 1 (items=21, used=993.3px, hwp_used≈1209.9px, diff=-216.6px)
```

`used / hwp_used / diff` 컬럼 정상 출력 — 페이지네이션 디버깅 가속화 도구로 즉시 활용 가능.

### CI (원본 브랜치)
| Check | 결과 |
|-------|------|
| CI / Build & Test | ✅ SUCCESS |
| CodeQL rust/js/python | ✅ 전부 SUCCESS |

## 충돌 분석

- **충돌 파일**: `mydocs/orders/20260425.md` 단 1개 (문서)
- **코드 충돌**: 없음
- **원인**: 오늘 머지된 PR #305 (Task #304) + Task #291 (메인테이너 핀셋) 섹션이 같은 위치 추가. PR 브랜치에는 Task #311/#312 섹션이 추가됨.
- **해결**: 모두 포함하는 형태로 메인테이너 직접 해결

## Investigation/Spike PR 패턴 평가

본 PR 은 어제 정착시킨 **Investigation/Spike PR 모범 사례**의 두 번째 사례:

1. **두 가설 모두 데이터로 부정** — vpos-reset 강제 분리, column 단일 origin
2. **부정 자체가 가치** — Epic #309 의 진로를 명확히 함
3. **의외의 발견** — TypesetEngine 이 이미 작동 중. 다음 단계 `#313` 으로 분리
4. **회귀 위험 0** — 모든 변경이 옵트인/진단 출력
5. **영구 자료** — `mydocs/tech/line_seg_vpos_analysis.md` 부록 A 추가

## 후속 PR 흐름

- **#316** (Epic #309 통합) — z-table 회귀로 보류 → **#320 (Task #318 회귀 수정)** 으로 분리됨
- **#319** (Task #317) — HWPX 어댑터 +1쪽 잔존 사안

본 PR (#315) 가 머지되면 #316 의 일부가 이미 반영된 상태가 되고, #320/#319 는 본 PR 위에 올라옴.

## 리스크 평가

| 리스크 | 판정 |
|--------|------|
| default 동작 변경 | ✅ 변경 0 (옵트인 + 진단 출력) |
| `PaginationOpts` 구조체 도입 시그니처 변경 | ✅ 호출 측 모두 마이그레이션 됨 |
| `used_height` 필드 추가 영향 | ✅ 기본값 0.0, 기존 동작 보존 |
| `dump-pages` 출력 형식 변경 | ✅ 기존 형식 유지하며 컬럼 추가만 |
| Golden SVG 회귀 | ✅ svg_snapshot 6 passed |

## 판정

✅ **Merge 권장**

**사유:**
1. **default 동작 변경 0** — 옵트인 플래그 + 진단 출력만 추가
2. **두 가설 모두 데이터 기반 부정** — 영구 자료로 보존되어 후속 작업자 진입 가속화
3. **의외의 발견** (TypesetEngine) — Epic #309 의 다음 단계 진로 명확
4. **CI + 로컬 검증 + 진단 도구 동작** 모두 통과
5. **CLAUDE.md 절차 완전 준수** — 두 task 모두 계획서/보고서 완비
6. Investigation/Spike PR 모범 사례 두 번째 적용

**Merge 전략:**
- orders 문서 충돌 메인테이너 직접 해결 완료
- `planet6897/task312` 에 push 후 admin merge

**WASM 시각 검증 불필요** — default 동작 변경 0이며 진단 도구만 추가. CLI 출력 검증으로 충분.
