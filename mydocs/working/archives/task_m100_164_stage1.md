# 단계별 완료 보고서 — Task #164 Stage 1

**이슈**: [#164](https://github.com/edwardkim/rhwp/issues/164)
**단계**: Stage 1 — 모듈 스켈레톤 + 빈 HWPX 생성
**브랜치**: `feature/task164-hwpx-serializer`
**완료일**: 2026-04-17

---

## 목표

Document IR을 HWPX(ZIP+XML)로 직렬화하는 모듈을 신설하고, 빈 문서를 한컴2020에서 정상 오픈할 수 있는 수준의 출력을 생성한다.

## 산출물

### 신규 파일

| 파일 | 설명 |
|------|------|
| `src/serializer/hwpx/mod.rs` | 엔트리 `serialize_hwpx()` + `HwpxSerializer` |
| `src/serializer/hwpx/writer.rs` | ZIP 컨테이너 래퍼 (`HwpxZipWriter`) |
| `src/serializer/hwpx/content.rs` | OPF manifest `Contents/content.hpf` 생성 |
| `src/serializer/hwpx/header.rs` | `Contents/header.xml` (Stage 1: 템플릿) |
| `src/serializer/hwpx/section.rs` | `Contents/section*.xml` (Stage 1: 템플릿) |
| `src/serializer/hwpx/static_assets.rs` | 정적 보일러플레이트 (version.xml, META-INF/*, settings.xml, Preview/*) |
| `src/serializer/hwpx/utils.rs` | XML escape / quick-xml 이벤트 헬퍼 |
| `src/serializer/hwpx/templates/` | 한컴2020 레퍼런스(ref_empty.hwpx) 기반 빈 문서 XML 5종 |
| `examples/hwpx_dump_empty.rs` | 빈 HWPX 파일 생성 유틸리티 |
| `samples/hwpx/ref/ref_{empty,text,table}.hwpx` | 한컴2020에서 생성한 레퍼런스 3종 |

### 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/mod.rs` | `SerializeError`를 루트로 이동 (HWP+HWPX 공용), `HwpxSerializer` 등록, `pub mod hwpx` 추가 |
| `src/serializer/cfb_writer.rs` | 중복 `SerializeError` 제거, `use super::SerializeError` 참조 |
| `src/error.rs` | `cfb_writer::SerializeError` → `serializer::SerializeError` 경로 수정 |

## 검증 결과

### 자동 테스트 (5/5 통과)

| 테스트 | 결과 |
|--------|------|
| `serialize_empty_doc_parses_back` | ✅ 0-섹션 라운드트립 |
| `serialize_with_one_section_parses_back` | ✅ 1-섹션 라운드트립 |
| `mimetype_is_first_entry` | ✅ ZIP 첫 엔트리 = mimetype |
| `mimetype_stored_not_deflated` | ✅ STORED(무압축) |
| `hancom_required_files_present` | ✅ 11개 필수 파일 검증 |

### 한컴2020 수동 검증

- **파일**: `output/stage1_empty.hwpx` (6946 bytes, 11개 ZIP 엔트리)
- **결과**: 한컴2020에서 **빈 1쪽 A4 문서로 정상 오픈** ✅
- 한컴 레퍼런스(ref_empty.hwpx)와 핵심 XML(header.xml 32731B, section0.xml 3340B) **바이트 동일**

### 한컴 레퍼런스 대비 차이 (허용 범위)

| 파일 | 한컴 ref | 우리 출력 | 사유 |
|------|---------|----------|------|
| PrvImage.png | 5126B (실제 썸네일) | 68B (1x1 투명 PNG) | 기능적 대체 |
| content.hpf | 1708B | 1624B | 사용자명·날짜 메타데이터 일반화 |
| 나머지 9개 | - | - | 바이트 동일 |

## 커밋 이력

1. `303cee9` — 수행·구현 계획서 작성
2. `2734c2a` — HWPX serializer 스켈레톤 + parse_hwpx 라운드트립
3. `b651075` — 한컴2020 레퍼런스 HWPX 3종 추가
4. `fec5540` — 한컴2020 호환 빈 HWPX 확장 (11개 필수 파일)
5. `1d831da` — 레퍼런스 XML 템플릿 임베딩으로 교체 (한컴 오픈 성공)

## Stage 2 준비 사항

Stage 2 "본문 문단·텍스트·lineSegArray"에서 교체할 항목:
- `header.rs`: 템플릿 → Document.doc_info(fonts/charShapes/paraShapes/styles) IR 기반 동적 생성
- `section.rs`: 템플릿 → Section.paragraphs/SectionDef IR 기반 동적 생성
- `content.rs`: 빈 문서 특례 → 모든 경우 동적 생성

레퍼런스 파일(ref_text.hwpx, ref_table.hwpx)을 Stage 2~3 구조 역공학 + 라운드트립 검증에 활용.

---

## 승인 요청

Stage 1 완료를 승인해주시면 **Stage 2 구현**을 시작하겠습니다.
