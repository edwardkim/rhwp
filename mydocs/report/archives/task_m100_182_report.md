# 최종 결과 보고서: HWPX Serializer 완성 — 표/이미지/스타일/글꼴 직렬화

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182` (→ `local/devel` merge 예정)
- **시작일**: 2026-04-17
- **완료일**: 2026-04-18
- **긴급도**: 최우선

## 1. 타스크 개요

### 배경

수식 개선·폰트 적용 등 다수 기능이 배포 대기 중이나, HWPX 저장 시 **표·이미지·스타일·글꼴 손실**로 v1.0.0 배포 불가 상태. 1단계 목표는 **"기존 HWPX 문서를 편집 후 저장했을 때 한컴2020이 온전히 다시 여는 것"** (작업지시자 지정).

### 전략 (Plan 모드 설계, 작업지시자 승인)

"이거 되면 저거 안 되는" 단편적 실패를 구조적으로 방지하기 위해 다음 8개 원칙을 채택:

1. Two-pass + `SerializeContext` (1-pass ID 풀 + 2-pass 일관 참조)
2. 빈 문서 특수 분기 제거 (거짓-양성 테스트 차단)
3. 누적 IrDiff 라운드트립 하네스 (Stage별 회귀 방지)
4. IR 의미 비교 (바이트 비교 대신)
5. 3-way BinData 단언 (`<hp:pic>` ↔ manifest ↔ ZIP)
6. #181(SVG 스냅샷)과 독립 진행
7. 한컴 OWPML 공식 오픈소스(Apache 2.0)를 스펙 참조원으로 활용
8. 한컴 DVC 검증기를 Stage 5 보조 게이트로 활용

### 외부 참조원 (신규 활용)

- [hancom-io/hwpx-owpml-model](https://github.com/hancom-io/hwpx-owpml-model) (Apache 2.0, © 2022 Hancom Inc.) — 292개 클래스, 2,245줄 enum 정의
- [hancom-io/dvc](https://github.com/hancom-io/dvc) (Apache 2.0, © 2022 Hancom Inc.) — HWPX 검증기

`THIRD_PARTY_LICENSES.md` 에 명시. `mydocs/tech/hwpx_hancom_reference.md`, `mydocs/tech/hwpx_dvc_reference.md` 에 상세 가이드 작성.

## 2. 단계별 수행 결과

| Stage | 주제 | 산출 모듈 | 완료일 |
|---|---|---|---|
| 0 | 기반 공사 | `context.rs`, `canonical_defaults.rs`, `roundtrip.rs`, `fixtures.rs`, `tests/hwpx_roundtrip_integration.rs` | 2026-04-17 |
| 1 | header.xml IR 동적화 | `header.rs` 13줄 → 760줄 | 2026-04-17 |
| 2 | section.xml `<hp:p>`/`<hp:run>` IR 연결 | `section.rs` 속성 동적화 | 2026-04-17 |
| 3 | Table 직렬화 | `table.rs` (430줄) | 2026-04-18 |
| 4 | Picture + BinData + 3-way 단언 | `picture.rs` (420줄) + `mod.rs` 확장 | 2026-04-18 |
| 5 | 도형·필드 + 대형 스모크 | `shape.rs` (290줄), `field.rs` (200줄) + 스모크 4건 | 2026-04-18 |

### 각 Stage 단계별 완료보고서

- `mydocs/working/task_m100_182_stage0.md` ~ `_stage5.md` (6개)

## 3. 검증 결과

### 3.1 단위 테스트

HWPX serializer 관련 57개 전부 통과:

| 모듈 | 테스트 수 |
|---|---|
| canonical_defaults | 5 |
| context | 5 |
| fixtures | 2 |
| roundtrip | 3 |
| header | 4 |
| section | 5 |
| table | 7 |
| picture | 6 |
| shape | 4 |
| field | 6 |
| mod (기존) | 11 (유지) |
| **합계** | **57** (Stage 0 대비 +47) |

### 3.2 통합 테스트

`tests/hwpx_roundtrip_integration.rs` 총 8개 통과:

- `stage0_blank_hwpx_roundtrip` (IrDiff 0)
- `stage1_ref_empty_roundtrip` (IrDiff 0)
- `stage1_ref_text_roundtrip` (IrDiff 0)
- `stage1_ref_mixed_header_level_regression_probe` (IrDiff 0)
- `stage5_ref_table_smoke` (크래시 없음)
- `stage5_form_002_smoke` (크래시 없음)
- `stage5_large_real_doc_2025_q1_smoke` (크래시 없음)
- `stage5_large_real_doc_2025_q2_smoke` (크래시 없음)

### 3.3 전체 라이브러리

**850 passed, 0 failed, 1 ignored** — 기존 783개 유지 + 신규 +47 (HWPX) + 기타 +20. **기존 기능 회귀 0건**.

### 3.4 참조 정합성 단언

- `SerializeContext::assert_all_refs_resolved()` — charPrIDRef/paraPrIDRef/borderFillIDRef/tabPrIDRef/numberingIDRef/styleID 전 참조 해소 확인
- `assert_bin_data_3way()` — `<hc:img binaryItemIDRef>` ↔ content.hpf `<opf:item>` ↔ ZIP entry 3 집합 동일성 단언

두 단언 모두 모든 라운드트립에서 통과.

## 4. 계획 vs 실제 차이

| 수행계획서 범위 | 실제 적용 | 이월 |
|---|---|---|
| Stage 0 기반 공사 | ✅ 완료 | — |
| Stage 1 header 동적화 | ✅ 완료 | — |
| Stage 2 section 동적화 (secPr/pagePr/grid 완전) | ⚠️ 축소 — `<hp:p>`/`<hp:run>` 속성만 IR 기반 | #186 범위 A/B |
| Stage 3 Table | ✅ 모듈 완성 | dispatcher 연결 #186 C |
| Stage 4 Picture + BinData | ✅ 모듈 완성 + 3-way 단언 | dispatcher 연결 #186 C |
| Stage 5 도형·필드 | ✅ 뼈대 모듈 완성 + 스모크 | dispatcher 연결 #186 C |

### 축소 근거 (작업지시자 지시)

- 1단계 목표를 **"기존 HWPX 편집·저장"** 으로 한정 (완전히 새 빈 문서 생성은 범위 외)
- SectionWriter 완전 동적화는 템플릿 교체 + Control dispatcher 통합 리팩토링이 필요 → **분할정복 원칙**으로 #186 으로 분리
- 각 Stage 의 `write_*` 함수는 독립 완성되어 검증됨 → hook 1~2줄로 연결 가능

### 이월 이슈

- **#186** (제목 갱신): `HWPX section.xml 완전 동적화 — secPr + 다중 run + Control dispatcher`
  - 범위 A: secPr/pagePr/grid 등 완전 동적화
  - 범위 B: `<hp:run>` 다중 분할 (`char_shapes[]` UTF-16 offset)
  - 범위 C: Control::Table/Picture/Shape/Field dispatcher 연결 (Stage 3/4/5 이월)
- **#185**: `rhwp validate CLI — 한컴 DVC Rust 포팅` (독립 후속)

## 5. 핵심 기술 결정

### 5.1 `SerializeContext` 의 양방향 ID 풀

```rust
pub struct IdPool<T> {
    registered: HashSet<T>,  // header 에서 정의
    referenced: HashSet<T>,  // section 에서 참조
}
```

1-pass 스캐너가 `doc.doc_info.char_shapes` 등에서 `register()` 호출, 2-pass 쓰기에서 `reference()` 호출. 종료 시 `registered - referenced` 가 공집합인지 단언. "section 에선 쓰는데 header 엔 없는" 상황이 **런타임에 확실히 잡힘**.

### 5.2 canonical_defaults.rs — 한컴 OWPML 기본값 60여 개

`Class/Head/CharShapeType.cpp:31`, `ParaShapeType.cpp:31` 등 한컴 C++ constructor 초기화 리스트에서 추출. **`snapToGrid=true` 하나만 예외**로 true 기본값, 나머지는 0/false/NONE.

### 5.3 3-way BinData 단언

```
ctx.bin_data_map ──▶ content.hpf <opf:item> 자동 전환
                 ╲
                  ╲─▶ ZIP entry BinData/imageN.ext 쓰기
                  ╱
<hp:pic>의 binaryItemIDRef = ctx.resolve_bin_id(bin_data_id)
```

한 출처에서 세 경로가 파생 → 구조적 일치 보장. 종료 직전 `assert_bin_data_3way()` 로 ZIP 엔트리 집합 vs ctx 집합 동일성 방어적 단언.

### 5.4 IrDiff 누적 확장

Stage 0에선 뼈대 필드(섹션 수·리소스 카운트)만 비교. Stage별로 `IrDifference` enum variant 추가하며 자동 확장. **삭제·완화 금지** — `is_empty()` 는 Stage 0~5 전부에 대해 하드 요구사항.

### 5.5 Canonical 속성·자식 순서

한컴 OWPML `WriteElement()`/`InitMap()` 근거로 다음 5개 클래스의 순서 명시·준수:

| 클래스 | 속성 순서 | 자식 순서 |
|---|---|---|
| charPr | id/height/textColor/shadeColor/useFontSpace/useKerning/symMark/borderFillIDRef | fontRef/ratio/spacing/relSz/offset/italic/bold/underline/... (15종) |
| paraPr | id/tabPrIDRef/condense/fontLineHeight/snapToGrid/suppressLineNumbers/checked | align/heading/breakSetting/margin/lineSpacing/border/autoSpacing |
| borderFill | id/threeD/shadow/centerLine/breakCellSeparateLine | slash/backSlash/leftBorder/rightBorder/topBorder/bottomBorder/diagonal/fillBrush |
| tbl | id/zOrder/numberingType/textWrap/textFlow/.../rowCnt/colCnt/cellSpacing/borderFillIDRef/noAdjust | sz/pos/outMargin/inMargin/tr[] |
| pic | id/zOrder/numberingType/textWrap/textFlow/.../href/groupLevel/instid/reverse | offset/orgSz/curSz/flip/rotationInfo/renderingInfo/imgRect/imgClip/inMargin/imgDim/img/effects/sz/pos/outMargin |

## 6. GitHub Discussions 등록

프로젝트 진행 중 발견한 인사이트를 커뮤니티에 공유:

- [Discussion #183](https://github.com/edwardkim/rhwp/discussions/183): HWPX 포맷 관찰 — 형식주의와 하이브리드 설계의 잔재
- [Discussion #184](https://github.com/edwardkim/rhwp/discussions/184): "LLM으로 HWPX 만들기" 성공 사례의 실체 — 3-way BinData 등 함정

## 7. 관련 문서

- 수행계획서: `mydocs/plans/task_m100_182.md`
- 구현계획서: `mydocs/plans/task_m100_182_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_182_stage{0..5}.md`
- OWPML 참조 가이드: `mydocs/tech/hwpx_hancom_reference.md`
- DVC 참조 가이드: `mydocs/tech/hwpx_dvc_reference.md`
- Plan 모드 산출: `~/.claude/plans/staged-foraging-naur.md`

## 8. 커밋 이력

- `4a4d99a` Task #182 Stage 0: 기반 공사
- `6c3983b` Task #182 Stage 1: header.xml IR 동적 생성
- `deabfc1` Task #182 Stage 2: section `<hp:p>`/`<hp:run>` IR 연결
- `7b56c92` Task #182 Stage 3: Table 직렬화 모듈
- `c1ca28a` Task #182 Stage 4: Picture + BinData + 3-way 단언
- (Stage 5 + 최종 보고서 커밋 예정)

## 9. 통계

| 항목 | 수치 |
|---|---|
| Stage 수 | 5 |
| 진행 일수 | 2일 (2026-04-17 ~ 04-18) |
| 신규 파일 | 9개 (context, canonical_defaults, fixtures, roundtrip, picture, shape, field, table, 통합 테스트) |
| 수정 파일 | 4개 (mod, header, section, content) |
| 새 단위 테스트 | 47개 |
| 새 통합 테스트 | 8개 |
| 라이브러리 전체 테스트 | 850 passed / 0 failed / 1 ignored |
| THIRD_PARTY_LICENSES.md 갱신 | hancom-io/hwpx-owpml-model + hancom-io/dvc 추가 |
| 기술문서 작성 | 2건 (OWPML 참조, DVC 참조) |
| Discussions 게시 | 2건 (#183, #184) |
| 파생 이슈 | #185 (DVC Rust 포팅), #186 (section 완전 동적화) |

## 10. 결론 및 다음 단계

### 달성한 것

1. **단편적 실패 방지 인프라** — `SerializeContext` + 3-way 단언 + IrDiff 하네스로 구조적 회귀 방지
2. **주요 HWPX 요소 9종 모듈화** — header, section, table, picture, shape, field, BinData, canonical_defaults, fixtures
3. **한컴 OWPML 공식 스펙 준수** — canonical 속성·자식 순서를 5개 핵심 클래스에 적용
4. **대형 실문서 4건 스모크 통과** — 파싱·직렬화·재파싱 크래시 없음
5. **기존 기능 회귀 0건** — 850/0/1

### 달성하지 못한 것 (이월)

1. **"완전히 새 빈 문서 생성"** — 1단계 범위 외, 2/3단계 목표
2. **SectionWriter 완전 동적화** — #186 범위 A/B
3. **Control dispatcher 연결** — #186 범위 C (Stage 3/4/5 이월)
4. **한컴2020 수동 검증** — Windows 환경 필요
5. **DVC 자동 검증** — Windows 환경 필요

### 다음 단계

1. Stage 5 + 최종 보고서 커밋
2. `local/task182` → `local/devel` merge (작업지시자 승인 필요)
3. `local/devel` → `devel` merge + push
4. 후속 이슈 착수 권장: **#186 → #185** (# 순서대로, Section 완전 동적화가 되어야 validate CLI 가 의미 있음)

## 11. 승인 요청

본 최종 결과 보고서 검토 후 승인 시:
- Stage 5 + 최종 보고서 커밋
- `local/devel` merge 준비
- 오늘할일 문서 `mydocs/orders/20260418.md` 에 완료 기록

피드백이 있으면 `mydocs/feedback/` 에 등록 부탁드립니다.
