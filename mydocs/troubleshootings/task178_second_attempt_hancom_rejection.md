# #178 두 번째 시도 — 한컴 호환 실패

- **일자**: 2026-04-19
- **이슈**: [#178](https://github.com/edwardkim/rhwp/issues/178)
- **브랜치**: `local/task178` (Stage 7 UI 변경 롤백, 어댑터 코드 보존)
- **결론**: **rhwp 자기 호환 부분 진척 + 한컴 호환 실패**. 본 타스크는 이번 배포에서 제외. 두 개의 후속 이슈로 분리.

## 1. 시도한 접근

첫 시도 (4587685, 폐기) 의 교훈에서 출발:
> HWPX-IR ↔ HWP-IR 매핑 어댑터 부재가 진짜 원인

본 시도는 **"잘 작동하는 HWP 직렬화기 어깨 위에 서자"** 정체성으로 7단계 어댑터 진행:

| Stage | 영역 |
|---|---|
| 1 | 진단 인프라 + 자동 차이 추출 도구 |
| 2 | `table.raw_ctrl_data` 합성 (CommonObjAttr 직렬화) |
| 3 | `cell.list_attr bit 16` (apply_inner_margin) |
| 4 | `Control::SectionDef` 컨트롤 삽입 + typeset.rs 버그픽스 |
| 5 | 통합 진입점 `export_hwp_with_adapter` + WASM 노출 |
| 6 | 명시적 검증 함수 `serialize_hwp_with_verify` |
| 7 | UI 복원 (file:save 분기 변경) — **롤백** |

## 2. rhwp 자기 호환 결과 (성공)

`samples/hwpx/hwpx-h-0[123].hwpx` → 어댑터 → HWP 직렬화 → `DocumentCore::from_bytes` 재로드:

| 샘플 | 원본 | Stage 1 (어댑터 미적용) | Stage 4+ (어댑터 적용) |
|---|---:|---:|---:|
| hwpx-h-01 | 9 | 200 | **9** ✅ |
| hwpx-h-02 | 9 | 220 | **9** ✅ |
| hwpx-h-03 | 9 | 224 | **9** ✅ |

→ rhwp 자기 직렬화 → 자기 재로드 페이지 수 100% 회복.

## 3. 한컴 호환 결과 (실패 — 이번 시도의 핵심 한계)

작업지시자가 rhwp-studio 에서 hwpx-h-01.hwpx → 저장 → 한컴2020 으로 열기:

> "전부 파일이 손상되었다고 열리지 않습니다."

3개 샘플 모두 한컴 거부.

## 4. 진단 — 한컴이 거부하는 영역들

### 4.1 SectionDef 결손 (가장 명확)

| 필드 | HWP 원본 (정상) | HWPX 출처 IR | 어댑터 후 |
|---|---|---|---|
| `flags` | `0x20000000` | `0x00000000` | **변경 없음** |
| `column_spacing` | `1134` | `0` | **변경 없음** |
| `raw_ctrl_extra` | **10바이트** | **0바이트** | **변경 없음** |
| `default_tab_spacing` | `8000` | `8000` | OK |

### 4.2 첫 문단 char_count vs text 불일치

| 필드 | HWP 원본 (정상) | HWPX 출처 IR |
|---|---|---|
| `text` | `'편집 탭 – 표'` (8자) | `''` (0자) |
| `char_count` | `25` | `25` (text 와 모순) |
| `has_para_text` | `true` | `true` |
| `controls.len()` | 2 (SectionDef + ColumnDef) | 4 (SectionDef + ColumnDef + Other + Table) |

`cell_split_save_corruption.md` 의 교훈과 정확히 같은 패턴 — `char_count` 와 PARA_TEXT 의 code unit 수가 불일치. 한컴이 파일 손상으로 판정.

### 4.3 추가 결손 가능성 (미진단)

- DocInfo 의 ID_MAPPINGS (FONT/CHAR_SHAPE/PARA_SHAPE/BORDER_FILL) 한컴 기대 형식
- FileHeader version / properties 비트
- Paragraph instanceId, raw_header_extra
- SECTION_DEF 의 추가 자식 레코드 (FOOTNOTE_SHAPE / PAGE_BORDER_FILL) 의 한컴 기대 값

## 5. 본 시도의 정체성 한계

**"잘 작동하는 직렬화기 어깨 위에 서자"** 는 직렬화기를 신뢰할 때만 성립. 본 시도가 부딪힌 진실:

> rhwp 의 HWP 직렬화기는 **HWP 출처 IR** 에 한해 한컴 호환을 보장한다.
> HWPX 출처 IR 은 한컴 기대치를 만족하지 못하는 영역이 광범위하고,
> 단순 어댑터 (필드 보강) 로는 메울 수 없다.

이는 첫 시도의 교훈 "안정적 경로 가정의 한계" 가 **rhwp 자기 검증** 에도 적용된다는 의미. **rhwp 자기 재로드 일치 ≠ 한컴 호환**.

## 6. 정리 결정

### 6.1 본 타스크 #178

- 이번 배포 (M100) 에서 **제외**
- Stage 7 의 UI 변경 (`rhwp-studio/src/command/commands/file.ts` + `hwpctl/index.ts`) **롤백** — HWPX 출처는 다시 HWPX 저장
- 어댑터 코드 (`src/document_core/converters/`) **보존** — 후속 이슈의 자산
- 진단 도구 (`examples/hwpx_hwp_ir_diff.rs`, `tests/hwpx_to_hwp_adapter.rs`) **보존**
- typeset.rs 버그픽스 **보존** — 독립적 버그픽스
- WASM `export_hwp` 자동 어댑터 분기 **PR 검토 시 결정**

### 6.2 후속 이슈 2건 (작업지시자 결정)

1. **HWPX 저장 사용자 고지** (M100, 이번 배포 포함) — rhwp-studio 가 HWPX 를 저장할 때 사용자에게 "HWPX 직접 저장은 한컴 호환 미보장" 모달/경고 표시. 어댑터 비활성, 사용자 책임 명시.

2. **HWPX→HWP 완전 변환기** (M101 또는 다음 패치) — 본격 매핑 구현. SectionDef 전체 필드 복원, DocInfo 검증, 첫 문단 text/char_count 동기화 등 광범위.

## 7. 본 시도에서 보존할 자산

| 자산 | 위치 | 후속 가치 |
|---|---|---|
| 진단 인프라 | `src/document_core/converters/diagnostics.rs` | 한컴 호환 진단 기반 |
| CommonObjAttr 작성기 | `src/document_core/converters/common_obj_attr_writer.rs` | 후속 변환기에서 재사용 |
| 어댑터 골격 | `src/document_core/converters/hwpx_to_hwp.rs` | 매핑 누적 컨테이너 |
| 통합 테스트 25개 | `tests/hwpx_to_hwp_adapter.rs` | 회귀 게이트 |
| typeset.rs 버그픽스 | `src/renderer/typeset.rs:1582-1588` | 독립 버그픽스 |
| 검증 함수 | `DocumentCore::serialize_hwp_with_verify` | 후속 검증 진입점 |
| hwp2hwpx 라이브러리 리뷰 | Stage 4 보고서 | 매핑 명세 권위 자료 |

## 8. 본 시도에서 얻은 교훈 (트러블슈팅의 핵심)

1. **rhwp 자기 검증의 한계** — `from_bytes(serialize(doc).page_count == doc.page_count` 가 한컴 호환을 의미하지 않는다. rhwp 파서는 자체 출력에 관대하지만 한컴은 엄격.

2. **"잘 작동하는 직렬화기 어깨 위" 의 함정** — 직렬화기 자체는 정상이어도 입력 IR (HWPX 출처) 이 직렬화기 가정을 만족 못 하면 결과는 손상. 직렬화기의 "정상" 은 입력 IR 모양에 종속.

3. **단순 어댑터의 한계** — 1:1 필드 매핑으로 메울 수 있는 영역과 의미적으로 채워야 하는 영역이 구분된다. 후자 (예: 첫 문단 text/char_count 동기화) 는 어댑터 범위를 넘는다.

4. **두 번째 실패의 가치** — 첫 시도는 "어댑터 부재" 를 발견. 두 번째 시도는 "어댑터로도 부족하다" 를 발견. 다음 시도 (#XXX 완전 변환기) 는 이 두 결론 위에서 시작.

5. **트러블슈팅 우선 검색** — `picture_save_hancom_compatibility.md` + `cell_split_save_corruption.md` + `table_paste_file_corruption.md` 을 본 시도 시작 시 읽었다면 한컴 호환 패턴을 더 빨리 인지했을 것. 직렬화·한컴 호환 관련 작업 전 트러블슈팅 폴더 전수 검색이 의무가 되어야 함.

## 9. 관련 자료

- 첫 시도: `mydocs/troubleshootings/task178_hwpx_to_hwp_first_attempt_failure.md`
- 본 시도 단계별 보고서: `mydocs/working/task_m100_178_stage[1..6].md`
- 본 시도 최종 보고서: `mydocs/report/task_m100_178_report.md`
- 참조 트러블슈팅: `picture_save_hancom_compatibility.md`, `cell_split_save_corruption.md`, `table_paste_file_corruption.md`
- 외부 라이브러리: `/home/edward/vsworks/hwp2hwpx` (hwplib/hwpxlib 저자, Apache 2.0)
- 디버그 샘플: `samples/hwpx/hwpx-h-0[123].hwpx`
