# 구현계획서 — Task M100-1201: HWPX masterpage idRef 기반 연결

## 설계 요약

현재 코드에는 HWPX masterpage 처리의 일부가 이미 들어 있다.

```text
src/parser/hwpx/content.rs
  - section_master_page_files를 manifest 순서 기반으로 추정

src/parser/hwpx/mod.rs
  - section index별 section_master_page_files를 읽어 SectionDef.master_pages에 push

src/parser/hwpx/section.rs
  - parse_hwpx_master_page() 존재
  - masterPage@type 일부 값(EVEN/ODD/LAST_PAGE/OPTIONAL_PAGE) 파싱
```

하지만 #1201의 본질은 HWPX section XML의 `<hp:masterPage idRef="...">`와 `content.hpf` manifest item id/href를 연결하는 것이다.
현재 방식은 section XML의 `idRef`를 읽지 않고 manifest 순서로 그룹을 추정하므로, manifest 순서가 달라지거나 masterpage가 section 앞뒤에 섞이면 잘못 연결될 수 있다.

이번 구현은 기존 masterpage parser를 살리되, 연결 방식을 `idRef -> manifest id -> href -> masterpage XML`로 바꾸는 데 집중한다.

## Stage 1 — manifest id map과 section idRef 수집

**목표**: HWPX masterpage 연결에 필요한 식별자 정보를 파서가 보존하게 한다.

변경 후보:

- `src/parser/hwpx/content.rs`
  - `PackageInfo`에 masterpage manifest 항목을 id와 href가 함께 보존되는 형태로 추가한다.
  - 후보 필드:

```rust
pub master_page_items: Vec<PackageItem>
```

  - 판별 기준은 기존과 같이 XML 항목 중 href가 `masterpage`를 포함하는 항목을 우선한다.
  - 기존 `section_master_page_files`는 fallback 또는 기존 테스트 호환 용도로 남긴다.

- `src/parser/hwpx/section.rs`
  - section XML에서 `<hp:masterPage idRef="...">`를 수집하는 read-only helper를 추가한다.
  - 후보 함수:

```rust
pub fn collect_hwpx_section_master_page_refs(xml: &str) -> Result<Vec<String>, HwpxError>
```

  - 네임스페이스 접두사 유무와 `Start`/`Empty` 이벤트를 모두 처리한다.
  - `idRef`가 없는 masterpage root는 수집하지 않는다.

테스트:

- `content.rs`
  - manifest의 `masterpage0 -> Contents/masterpage0.xml` id/href 보존 테스트
  - 기존 manifest 순서 기반 fallback 테스트 유지
- `section.rs`
  - `<hp:masterPage idRef="masterpage0"/>`, `<masterPage idRef="masterpage1"/>` 수집 테스트
  - 일반 section 문단 파싱과 독립적으로 동작하는지 확인

보고서:

- `mydocs/working/task_m100_1201_stage1.md`

## Stage 2 — idRef 기반 masterpage 연결

**목표**: `parse_hwpx()`가 section별 masterpage를 manifest 순서 추정이 아니라 section의 `idRef` 기준으로 연결하게 한다.

변경 후보:

- `src/parser/hwpx/mod.rs`
  - `PackageInfo.master_page_items`로 id-to-href map을 만든다.
  - section XML을 읽은 뒤 `collect_hwpx_section_master_page_refs()`로 참조 id 목록을 얻는다.
  - 각 `idRef`를 manifest id에 매칭해 해당 masterpage XML을 읽고 `parse_hwpx_master_page()`로 변환한다.
  - 변환 결과를 `section.section_def.master_pages`에 push한다.
  - 같은 href가 중복 연결되지 않도록 section 단위 dedup을 적용한다.
  - `idRef`가 없거나 모두 resolve되지 않을 때만 기존 `section_master_page_files` fallback을 사용한다.

주의:

- HWP5 raw parser의 `[Both, Odd, Even]` 순서 해석은 건드리지 않는다.
- HWPX에서는 XML root의 `type` 속성을 신뢰한다.
- 누락된 idRef나 masterpage 파일은 기존처럼 warning으로 처리하고 문서 전체 파싱은 계속한다.

테스트:

- pure helper를 추가할 수 있으면 `idRef -> href` resolve 테스트를 단위 테스트로 둔다.
- full HWPX ZIP fixture가 과해지면 작은 내부 helper 중심으로 테스트하고, 실제 샘플 검증은 Stage 4에서 수행한다.

보고서:

- `mydocs/working/task_m100_1201_stage2.md`

## Stage 3 — masterPage@type 파싱 정합성 보강

**목표**: 공식 문서 표기와 실제 샘플 표기를 모두 안정적으로 해석한다.

변경 후보:

- `src/parser/hwpx/section.rs`
  - `parse_master_page_start()`의 `type` 해석을 보조 함수로 분리한다.
  - `BOTH`, `Both`, `both` -> `HeaderFooterApply::Both`
  - `EVEN`, `Even`, `even` -> `HeaderFooterApply::Even`
  - `ODD`, `Odd`, `odd` -> `HeaderFooterApply::Odd`
  - `LAST_PAGE`, `LastPage`, `lastPage` -> extension + `Both`
  - `OPTIONAL_PAGE`, `OptionalPage`, `optionalPage` -> extension + `Both`

주의:

- `LAST_PAGE pageDuplicate="0"`의 `replace_base` 및 `overlap` 보정은 유지한다.
- `raw_list_header` 생성 방식은 기존 저장 contract와 충돌하지 않게 유지한다.

테스트:

- `parse_hwpx_master_page()`에 mixed-case type 테스트 추가
- 기존 `LAST_PAGE`/`OPTIONAL_PAGE` 테스트 유지

보고서:

- `mydocs/working/task_m100_1201_stage3.md`

## Stage 4 — 대상 샘플 구조/시각 검증

**목표**: 실제 #1201 샘플에서 section별 바탕쪽이 채워지고 홀짝이 반전되지 않는지 확인한다.

대상:

```text
/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx
/Users/melee/Downloads/[2027] 온새미로 1 본교재.pdf
```

검증 후보:

```text
cargo run -- dump "/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx"
cargo run -- export-svg "/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1201_masterpage -p 3
cargo run -- export-svg "/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1201_masterpage -p 4
```

확인 항목:

- section0~section4의 `master_pages`가 기대 개수로 채워지는가
- `masterpage0/2/4/6/8`의 EVEN이 짝수쪽에 적용되는가
- `masterpage1/3/5/7/9`의 ODD가 홀수쪽에 적용되는가
- PDF 기준 4쪽과 5쪽에서 바탕쪽 위치와 텍스트 방향이 반전되지 않는가

보고서:

- `mydocs/working/task_m100_1201_stage4.md`

## Stage 5 — 회귀 검증과 최종 보고

필수 검증:

```text
cargo fmt --all --check
cargo test --lib hwpx
cargo test --test issue_1100_exam_social_hwpx_header
cargo test --test issue_1113_header_autonum_placeholder
```

필요 시 확장 검증:

```text
cargo test --lib
cargo test --test hwpx_roundtrip_integration
```

최종 보고:

- `mydocs/report/task_m100_1201_report.md`

## 제외 범위

- #1196 맞쪽/제본 여백 문제
- #1197 4쪽 이미지와 텍스트 겹침
- #1205 문단 박스 borderFill `NONE` 처리
- 렌더러의 홀짝 선택 알고리즘 전면 변경
- 대상 원본 HWPX/PDF 파일의 저장소 커밋 포함

## 완료 판정

완료 조건:

- section XML의 `masterPage idRef`가 파싱된다.
- manifest id/href와 `idRef`가 연결된다.
- `SectionDef.master_pages`가 대상 샘플의 section별 바탕쪽을 보존한다.
- masterpage `type`은 HWPX XML의 명시값을 기준으로 매핑된다.
- 실제 샘플 4쪽/5쪽에서 PDF 기준 홀짝 바탕쪽이 반전되지 않는다.
- 기존 HWP5 바탕쪽 파싱과 HWPX header/footer 관련 회귀 테스트가 통과한다.

## 작업지시자 승인 요청

본 구현계획이 승인되면 Stage 1부터 소스 수정을 시작한다.
첫 수정 범위는 `src/parser/hwpx/content.rs`와 `src/parser/hwpx/section.rs`의 read-only 수집 경로로 제한한다.
