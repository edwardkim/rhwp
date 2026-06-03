# Task #164 최종 결과 보고서 — HWPX Serializer

**이슈**: [#164](https://github.com/edwardkim/rhwp/issues/164) — HWPX serializer (Document IR → .hwpx)
**브랜치**: `feature/task164-hwpx-serializer` (12 commits)
**기간**: 2026-04-17
**마일스톤**: M100 (v1.0.0)

---

## 목표

`Document` IR을 한글2020/한컴오피스가 정상 오픈할 수 있는 `.hwpx` (HWPML 패키지) 파일로 직렬화하는 기능을 구현한다.

## 산출물

### 1. 모듈 구조

`src/serializer/hwpx/`:
| 파일 | 역할 |
|------|------|
| `mod.rs` | `HwpxSerializer` + `serialize_hwpx()` 진입점 |
| `writer.rs` | ZIP 작성 (mimetype STORED, 나머지 DEFLATED) |
| `header.rs` | `Contents/header.xml` (현재 레퍼런스 템플릿 임베딩) |
| `section.rs` | `Contents/section{N}.xml` 동적 생성 (다문단/탭/소프트브레이크) |
| `content.rs` | `Contents/content.hpf` OPF manifest |
| `static_assets.rs` | mimetype/version/META-INF/Preview 정적 자산 |
| `utils.rs` | XML 헬퍼 + escape |
| `templates/*.xml` | 한컴2020 레퍼런스 기반 정적 템플릿 |

### 2. 지원하는 IR 매핑

| IR | HWPX | 비고 |
|----|------|------|
| `Document` | ZIP 패키지 (한컴 11개 필수 파일) | mimetype STORED 1순위 |
| `Section` | `Contents/section{N}.xml` | 첫 섹션은 `<hp:secPr>` 포함 |
| `section.paragraphs` 여러 개 | `<hp:p>` 여러 개 (하드 문단 경계) | |
| `paragraph.text` | `<hp:t>` 텍스트 (XML escape) | |
| `paragraph.text` 내 `\n` | `<hp:lineBreak/>` (소프트 브레이크) | 같은 문단 내 |
| `paragraph.text` 내 `\t` | `<hp:tab width="4000" leader="0" type="1"/>` | |
| `Document.bin_data_content` 부재 시 | content.hpf 레퍼런스 템플릿 사용 | |

### 3. 단계별 진행

| Stage | 내용 | 보고서 |
|-------|------|--------|
| 1 | 모듈 스켈레톤 + 한컴 호환 11파일 빈 HWPX | `task_m100_164_stage1.md` |
| 2.1 | section.xml 텍스트 IR 주입 + xml_escape | `task_m100_164_stage2_1.md` |
| 2.2 | 탭/줄바꿈 인라인 직렬화 (구조만) | `task_m100_164_stage2_2.md` |
| 2.3 | 다문단 + 소프트 브레이크 + 탭 정식 (한컴 레퍼런스 ref_mixed.hwpx 역공학) | `task_m100_164_stage2_3.md` |

### 4. 검증 인프라

| 도구 | 역할 |
|------|------|
| `tools/verify_hwpx.py` | 한컴 OLE(pyhwpx) 기반 단일 파일 검증 |
| `tools/verify_all.py` | 모든 stage 산출물 일괄 자동 검증 |
| `examples/hwpx_dump_empty.rs` | 빈 HWPX 생성 |
| `examples/hwpx_dump_text.rs` | 텍스트/탭/소프트브레이크 산출 |
| `examples/hwpx_roundtrip.rs` | 입력 HWPX → 파싱 → 재직렬화 |

## 검증 결과

### 단위 테스트 — 10/10 통과
- `serialize_empty_doc_parses_back`
- `serialize_with_one_section_parses_back`
- `serialize_text_paragraph_roundtrip`
- `tab_and_linebreak_emitted_inline`
- `linesegs_emitted_per_linebreak`
- `multi_paragraph_emits_multiple_hp_p`
- `xml_escape_applied_to_section_text`
- `mimetype_is_first_entry`
- `mimetype_stored_not_deflated`
- `hancom_required_files_present`

### 한글2020 자동 검증 — 4/4 통과
- `stage1_empty.hwpx` — 빈 1쪽 A4
- `stage2_text.hwpx` — "안녕 Hello 123"
- `stage2_mixed.hwpx` — 4문단 + 소프트 브레이크 + 탭
- `rt_ref_mixed.hwpx` — ref_mixed.hwpx 라운드트립

### 실문서 라운드트립 (참고용)
- "2025년 2분기 해외직접투자 (최종).hwpx" (134KB, 130문단, 2섹션)
  - 한글2020 정상 오픈 ✅
  - 본문 텍스트 보존 ✅
  - 3페이지 페이지네이션 동작 ✅
  - 표/이미지/스타일은 범위 밖 (Stage 2.5+)

## 한계 및 향후 작업

### 현재 미지원 (의도적 범위 외)

1. **글꼴/스타일 IR 직렬화** — `header.xml`이 레퍼런스 템플릿 고정.
   `Document.doc_info.fonts/char_shapes/para_shapes/styles` IR을 OWPML 속성으로 매핑 필요.
   → **Task #165 제안**: "header.xml IR 기반 직렬화 (글꼴/스타일/문단 모양)"

2. **표/이미지/도형** — `Control` IR 직렬화 미구현.
   → **Task #166 제안**: "Section 컨트롤(표/이미지/그림) HWPX 직렬화"

3. **소프트 브레이크 라운드트립의 IR 표현** — 현재 `paragraph.text`에 `\n`이 들어오면 소프트 브레이크로 처리하지만, HWPX 파서가 `\n`로 보존하는지 별도 검증 필요.

4. **paraPrIDRef/charPrIDRef 동적 매핑** — 현재 모두 `"0"` 고정.
   header.xml IR화 이후 IR의 `style_id`/`char_shape_id`와 연동 필요.

### 외부 의존성 (개선 가능)

- `verify_hwpx.py`는 Windows + 한컴오피스 + `pyhwpx` 필요. CI에서는 SVG 스냅샷 기반 검증으로 대체 가능 (Task #167 제안).

## 커밋 이력 (12개)

```
7c76035 Task #164: HWPX 라운드트립 예제 + 회귀 케이스 추가
5325c02 Task #164: 회귀 테스트 배치 스크립트 추가
ebe5f2b Task #164: HWPX 자동 검증 도구 추가
edeaa1e Task #164 [Stage 2.3]: 다문단 + 소프트 브레이크 + 탭 정식 직렬화
0412834 Task #164 [Stage 2.2]: 탭/줄바꿈 인라인 직렬화
0a0f14e Task #164 [Stage 2.1]: section.xml 텍스트 IR 주입
1c0fb9a Task #164 [Stage 1]: 단계별 완료 보고서
1d831da Task #164 [Stage 1]: 한컴2020 레퍼런스 XML 템플릿 임베딩으로 교체
fec5540 Task #164 [Stage 1]: 한컴2020 호환 빈 HWPX 확장 (11개 필수 파일)
b651075 Task #164 [Stage 1]: 한글2020에서 생성한 레퍼런스 HWPX 3종 추가
2734c2a Task #164 [Stage 1]: HWPX serializer 스켈레톤 + 빈 Document 라운드트립
303cee9 Task #164: 수행·구현 계획서 작성
```

## 결론

Task #164의 핵심 목표 — "Document IR → 한컴오피스가 오픈 가능한 HWPX 생성" — 을 텍스트 위주 시나리오 기준으로 완료했다. 다문단·소프트 라인브레이크·탭이 정확히 동작하며, 사용자 제공 레퍼런스(`ref_mixed.hwpx`)와 134KB 실문서 라운드트립이 한글2020에서 검증 통과했다.

후속 작업(글꼴/스타일/표/이미지)은 별도 이슈로 분리 추적하는 것을 권장한다.
