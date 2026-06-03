# #178 최종 보고서 — HWPX→HWP IR 매핑 어댑터 (정도 접근)

- **타스크**: [#178](https://github.com/edwardkim/rhwp/issues/178)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task178`
- **기간**: 2026-04-19 (단일일 진행)
- **상태**: **부분 진척 + 한컴 호환 실패** — 이번 배포에서 제외, 후속 이슈 2건으로 분리

## 1. 요약

본 타스크 두 번째 시도는 "잘 작동하는 HWP 직렬화기 어깨 위에 서자" 정체성으로 7단계 어댑터를 진행했다. **rhwp 자기 호환은 100% 회복 (페이지 9 보존) 했으나 한컴 호환은 실패 (전 샘플 거부)**.

본 타스크는 이번 배포에서 제외하고, 다음 두 이슈로 분리:
- **HWPX 저장 사용자 고지** (M100, 이번 배포)
- **HWPX→HWP 완전 변환기** (다음 패치/릴리즈)

상세 트러블슈팅: `mydocs/troubleshootings/task178_second_attempt_hancom_rejection.md`

## 2. 진척 측정

### 2.1 rhwp 자기 호환 (성공)

| 샘플 | 원본 페이지 | HWP 저장 후 재로드 |
|---|---:|---:|
| hwpx-h-01 | 9 | **9** ✅ |
| hwpx-h-02 | 9 | **9** ✅ |
| hwpx-h-03 | 9 | **9** ✅ |

### 2.2 한컴 호환 (실패)

작업지시자 보고: "전부 파일이 손상되었다고 열리지 않습니다."

## 3. 수행한 작업

### 3.1 단계별 산출물

| Stage | 산출물 | 보고서 |
|---|---|---|
| 1 | 진단 인프라 + `examples/hwpx_hwp_ir_diff.rs` CLI + 베이스라인 측정 | [stage1](mydocs/working/task_m100_178_stage1.md) |
| 2 | `common_obj_attr_writer.rs` + `table.raw_ctrl_data` 합성 | [stage2](mydocs/working/task_m100_178_stage2.md) |
| 3 | `cell.list_attr bit 16` 보강 | [stage3](mydocs/working/task_m100_178_stage3.md) |
| 4 | `Control::SectionDef` 컨트롤 삽입 + `typeset.rs` 버그픽스 | [stage4](mydocs/working/task_m100_178_stage4.md) |
| 5 | `export_hwp_with_adapter` 통합 진입점 + WASM 노출 | [stage5](mydocs/working/task_m100_178_stage5.md) |
| 6 | `serialize_hwp_with_verify` 명시 검증 함수 | [stage6](mydocs/working/task_m100_178_stage6.md) |
| 7 | UI 변경 → **롤백** (한컴 호환 실패 확정) | (본 보고서) |

### 3.2 코드 변경 요약

| 영역 | 보존 (Yes/No) |
|---|---|
| `src/document_core/converters/` (어댑터 + 진단 + 작성기) | **Yes** — 후속 이슈 자산 |
| `tests/hwpx_to_hwp_adapter.rs` (통합 테스트 25개) | **Yes** |
| `examples/hwpx_hwp_ir_diff.rs` (CLI) | **Yes** |
| `src/renderer/typeset.rs:1582-1588` (버그픽스) | **Yes** — 독립 버그픽스 |
| `src/document_core/commands/document.rs` (진입점 + 검증 함수) | **Yes** |
| `src/wasm_api.rs` (자동 어댑터 분기) | **PR 검토 시 결정** |
| `rhwp-studio/src/command/commands/file.ts` (UI 분기) | **No** — 롤백 |
| `rhwp-studio/src/hwpctl/index.ts` (SaveAs) | **No** — 롤باк |

### 3.3 검증

| 항목 | 결과 |
|---|---|
| 단위 테스트 (converters) | 16개 ✅ |
| 통합 테스트 (어댑터) | 25개 ✅ |
| 라이브러리 전체 회귀 | 891개 ✅ |
| HWP 직렬화기 수정 | 0줄 (정체성 준수) ✅ |
| WASM 빌드 | 성공 ✅ |
| **한컴 수동 검증** | **❌ 실패** |

## 4. 핵심 발견 4건

### 4.1 페이지 폭주의 진짜 원인 (Stage 4)

HWPX 파서는 `Section.section_def` 만 채우고 `Control::SectionDef` 컨트롤을 `paragraph.controls` 에 삽입하지 않음. HWP 직렬화기가 PAGE_DEF 출력 못 함 → 재로드 시 page_def 가 모두 0 → 169 페이지 폭주.

### 4.2 typeset.rs:1582 버그 발견 (Stage 4)

`get_table_vertical_offset` 가 paginator 와 다른 식 (`raw_ctrl_data[0..4]` = attr 비트). HWPX 출처 (raw_ctrl_data 비어있음) 에서는 0 반환으로 영향 없었으나, 어댑터가 raw_ctrl_data 를 채우면서 활성화. 본 타스크에서 paginator 와 통일.

### 4.3 vpos 사전계산 불필요 (작업지시자 통찰)

`reflow_zero_height_paragraphs` 가 HWPX 로드 시점에 IR 의 vpos 를 in-place 갱신. 어댑터에서 별도 계산 불필요. 작업지시자 통찰로 167줄 절감.

### 4.4 rhwp 자기 검증의 한계 (Stage 7 — 핵심 교훈)

`from_bytes(serialize(doc)).page_count == doc.page_count` 는 한컴 호환을 의미하지 않는다. rhwp 파서는 자체 출력에 관대하지만 한컴은 엄격. 다음 영역에서 거부:
- SectionDef `flags / column_spacing / raw_ctrl_extra` 결손
- 첫 문단 `text / char_count` 동기화 실패 (cell_split_save_corruption.md 와 동일 패턴)
- DocInfo / FileHeader 미진단 영역들

## 5. 후속 이슈 (작업지시자 결정)

### 5.1 HWPX 저장 사용자 고지 (M100, 이번 배포)

rhwp-studio 가 HWPX 를 저장할 때 사용자 경고 — "HWPX 직접 저장은 한컴 호환 미보장. 백업 권장." 어댑터 비활성, 사용자 책임.

### 5.2 HWPX→HWP 완전 변환기 (M101 또는 다음 패치)

본격 매핑 구현. SectionDef 전체 필드 복원, DocInfo 검증, 첫 문단 text/char_count 동기화 등 광범위. 본 타스크의 어댑터 자산을 기반으로 시작.

## 6. 본 타스크에서 보존할 자산 가치

| 자산 | 가치 |
|---|---|
| 진단 인프라 + CLI | 후속 이슈의 한컴 호환 진단 기반 |
| `serialize_common_obj_attr` (158줄) | 후속 변환기에서 그대로 재사용 |
| 어댑터 골격 + idempotent 보장 | 매핑 누적 컨테이너 |
| 통합 테스트 25개 | 회귀 게이트 (확장 가능) |
| `typeset.rs` 버그픽스 | 독립 버그픽스 (한컴 호환과 무관하게 가치) |
| `serialize_hwp_with_verify` | 후속 검증 진입점 |
| hwp2hwpx (Java) 라이브러리 리뷰 | 매핑 명세 권위 자료 (Stage 4 보고서) |

## 7. 일정·리소스 회고

- **추정 일정**: 6~9일
- **실제 진행**: 단일일 (Stage 1~6 + Stage 7 부분 + 한컴 검증 + 정리)
- **결과**: 한컴 호환 실패로 본 타스크 미완성, 후속 이슈로 분리

원래 일정 (6~9일) 추정이 빠른 진행으로 단축됐지만 결과는 미완성. 단순 어댑터 범위로는 한컴 호환 불가능함을 단일일 내에 확인한 점은 진척. 첫 시도와 마찬가지로 **수일에 걸쳐 폭주만 키우는 대신 빠른 실패 → 정리 → 후속 이슈 분리** 패턴 유지.

## 8. 정체성 회고

본 타스크 정체성 ("잘 작동하는 HWP 직렬화기 어깨 위에 서자") 은 **HWP 출처 IR 에 한해 성립**. HWPX 출처 IR 은 HWP 직렬화기의 가정을 충족하지 못하는 영역이 광범위하므로 정체성 자체가 부분적으로 잘못 설정됐음.

다음 시도의 정체성은 "**HWPX→HWP 완전 변환기 (한컴 정상 파일 패턴 모방)**" 가 더 정확. 단순 어댑터가 아니라, HWP 출처 IR 과 동등한 IR 을 만드는 본격 변환.

## 9. 승인 요청

본 최종 보고서 + 트러블슈팅 + Stage 7 UI 롤백 + 후속 이슈 2건 등록 진행에 대한 승인 요청.

승인 후:
1. 본 변경 사항 커밋
2. `local/task178` → `local/devel` merge
3. `local/devel` → `origin/devel` push
4. 후속 이슈 2건 GitHub 등록
5. 이슈 #178 close 코멘트 (본 보고서 + 후속 이슈 링크)
