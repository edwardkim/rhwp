# 최종 결과 보고서 — Task M100 #416

## 이슈

[#416](https://github.com/edwardkim/rhwp/issues/416) — find_bin_data 가드 결함 (Task #195 회귀, 페이지 배경 이미지 잘못 표시)

## 결과 요약

`src/renderer/layout/utils.rs::find_bin_data` 함수의 가드 `c.id == bin_data_id` 를 제거하여, **storage_id ≠ 인덱스인 모든 hwp 파일에서 이미지가 정상 매칭** 됨.

`samples/hwpspec.hwp` 1 페이지의 페이지 표지 이미지가 정상 PNG (BIN000C, 1137 bytes) 로 매칭 — 작업지시자 시각 판정 통과.

## 회귀 origin

| 항목 | 값 |
|------|-----|
| 회귀 도입 커밋 | [`5c72f48`](https://github.com/edwardkim/rhwp/commit/5c72f48) (Task #195 OOXML 차트, @planet6897, 2026-04-20) |
| 발견 시점 | 2026-04-28 (작업지시자 시각 검증) |
| 본질 | 트러블슈팅 [`bin_data_id_index_mapping.md`](mydocs/troubleshootings/bin_data_id_index_mapping.md) 의 정정 결함 패턴이 fallback 가드 추가로 재발 |
| 메인테이너 책임 | PR #195 검토에서 트러블슈팅 문서 참조 누락 — 동일 결함 패턴 재발 인지 못함 |

## 변경 파일

`src/renderer/layout/utils.rs` 단일 파일.

### 변경 1 — `find_bin_data` 함수 본체 (line 13-25)

```diff
 pub(crate) fn find_bin_data<'a>(bin_data_content: &'a [BinDataContent], bin_data_id: u16) -> Option<&'a BinDataContent> {
     if bin_data_id == 0 {
         return None;
     }
-    // 1-indexed 순번으로 먼저 조회 (기존 동작 유지)
+    // 1-indexed 순번으로 BinDataContent 배열 접근
     if let Some(c) = bin_data_content.get((bin_data_id - 1) as usize) {
-        if c.id == bin_data_id {
-            return Some(c);
-        }
+        return Some(c);
     }
-    // 실패 시 id 필드로 직접 검색 (HWPX 차트처럼 sparse id 사용 시)
+    // 인덱스 범위 밖 (HWPX 차트 sparse id 60000+N 등) — id 직접 검색
     bin_data_content.iter().find(|c| c.id == bin_data_id)
 }
```

함수 docstring 에 정정 사유 + 트러블슈팅 문서 참조 추가.

### 변경 2 — 단위 테스트 7 개 추가

`#[cfg(test)] mod tests` 신규:

| 테스트 | 검증 |
|---|---|
| `find_bin_data_returns_none_for_zero` | bin_data_id=0 → None |
| `find_bin_data_indexed_match_storage_id_differs` | hwpspec 패턴 — id=12 가 bin_data_id=1 매칭 |
| `find_bin_data_indexed_match_storage_id_matches` | 일반 케이스 — storage_id == 인덱스 |
| `find_bin_data_sparse_id_for_hwpx_chart` | HWPX 차트 60001/60002 sparse id |
| `find_bin_data_out_of_range_returns_none` | 인덱스 범위 밖 + 일치 id 없음 → None |
| `find_bin_data_hwpx_realistic_layout_with_chart` | HWPX 일반 BinData + 차트 sparse id 혼합 layout |
| `find_bin_data_hwp_hwpspec_page_bg_pattern` | hwpspec.hwp 14 BinData 모사 |

### 변경 3 — 트러블슈팅 문서 갱신

`mydocs/troubleshootings/bin_data_id_index_mapping.md`:
- **회귀 이력** 섹션 추가 (2026-04-20 회귀 origin + 2026-04-28 재정정)
- **추가 교훈** 추가 (가드/fallback 추가 시 트러블슈팅 정독, PR 검토 절차 강화, 정정과 함께 단위 테스트 필수)

## 단계별 진행

| 단계 | 작업 | 커밋 | 보고서 |
|------|------|------|--------|
| Stage 1 | `find_bin_data` 가드 제거 + 단위 테스트 5 개 | `feb0774` | `mydocs/working/task_m100_416_stage1.md` |
| Stage 2 | 차트 회귀 방지 단위 테스트 2 개 + hwpspec 시각 검증 | `ca44b01` | `mydocs/working/task_m100_416_stage2.md` |
| Stage 3 | 자동 검증 종합 + 트러블슈팅 문서 갱신 + WASM 빌드 | (본 커밋) | `mydocs/working/task_m100_416_stage3.md` |

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1023 passed**, 0 failed (1016 → +7 신규) |
| `cargo test --lib renderer::layout::utils::` | ✅ 7/7 passed |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 20s, 4,100,250 bytes (-4,672 bytes) |
| `samples/hwpspec.hwp` 1 페이지 시각 검증 | ✅ 페이지 표지 정상 |
| 작업지시자 시각 판정 | ✅ 통과 |

### 차트 회귀 위험 점검

| 케이스 | Task #195 (가드) | 본 task (가드 제거) | 동작 |
|--------|----------------|-------------------|------|
| HWP 일반 그림, storage_id ≠ 인덱스 (hwpspec) | fallback storage_id 검색 → 잘못된 매칭 | **인덱스 매칭** (정확) | 정정 |
| HWP/HWPX 일반 그림 (storage_id == 인덱스) | 인덱스 매칭 | 인덱스 매칭 | 동일 |
| HWPX 차트 (id=60001+) | 인덱스 None → fallback id 검색 → 매칭 | 인덱스 None → fallback id 검색 → 매칭 | **동일** |

→ 차트 회귀 위험 0 (sparse id 60001+ 는 항상 인덱스 범위 밖).

## 시각 검증 중 발견된 별개 결함 (별도 이슈로 처리)

작업지시자가 `samples/hwpspec.hwp` **20 페이지** 검증 중 **이미지 이중 출력** 결함 발견:

- 같은 이미지가 y 좌표 약 2.67px 차이로 두 번 그려짐 (3 쌍, 총 6 개 image 요소)
- **devel 시점 (Task #416 정정 전)** 에도 4 개 image 가 있었음 — 본 task 의 정정과 별개로 이미 존재하던 회귀
- 본 task 정정의 부수효과로 4 → 6 으로 늘어났을 가능성 (정확 매칭 후 일부 안 그려지던 이미지가 추가로 그려짐)
- **별도 이슈로 등록** 후 별도 task 로 처리 — Task #416 은 본 결함 (페이지 표지) 만으로 마무리

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 트러블슈팅 폴더 사전 검색 의무 | ✅ `bin_data_id_index_mapping.md` 검색 → 동일 결함 패턴 인지 |
| 작업지시자 시각 판정 게이트 | ✅ hwpspec.hwp 1 페이지 시각 판정 통과 후 진행 |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ 가드 제거 — 일반화된 검증 가드가 오히려 회귀 유발한 사례 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/task416` 에서 커밋 |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 커밋
2. `local/task416` → `local/devel` → `devel` 머지 + push
3. 이슈 #416 close
4. **별도 이슈** — 페이지 20 이미지 이중 출력 결함 등록

## 산출물

- 변경 파일: `src/renderer/layout/utils.rs`
- 트러블슈팅 갱신: `mydocs/troubleshootings/bin_data_id_index_mapping.md`
- WASM 빌드: `pkg/rhwp.js`, `pkg/rhwp_bg.wasm`
- 시각 산출물: `output/svg/issue-416/hwpspec_001.svg`
- 수행 계획서: `mydocs/plans/task_m100_416.md`
- 구현 계획서: `mydocs/plans/task_m100_416_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_416_stage{1,2,3}.md`
- 최종 보고서: `mydocs/report/task_m100_416_report.md` (본 문서)

## 참고

- 이슈: [#416](https://github.com/edwardkim/rhwp/issues/416)
- 브랜치: `local/task416`
- 회귀 origin 커밋: [`5c72f48`](https://github.com/edwardkim/rhwp/commit/5c72f48) (Task #195)
- 트러블슈팅: `mydocs/troubleshootings/bin_data_id_index_mapping.md` (2026-02-17 최초 + 2026-04-28 회귀 정정)
- 작업 일자: 2026-04-28
