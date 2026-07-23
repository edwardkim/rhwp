# 완료 보고서 — Task M100-3195

- 이슈: #3195
- 제목: [HWP5 저장] SectionDef page_num_type(홀/짝 쪽번호 시작)이 flags에 미반영되어 저장 시 유실
- 작성일: 2026-07-23
- 브랜치: `fix/3195-section-def-page-num-type-flags-sync`

## 1. 문제

HWPX 파서(`src/parser/hwpx/section.rs::parse_start_num`)는 `<hp:startNum
pageStartsOn="ODD|EVEN">` 을 읽어 `SectionDef.page_num_type` 필드만 세팅하고
`flags` 비트 20-21은 건드리지 않는다. 반면 HWP5는 `page_num_type`을 별도
필드로 갖지 않고 `flags` 비트 20-21로만 표현하며, HWP5 직렬화기
(`src/serializer/control.rs::serialize_section_def`)는 `sd.flags`를 그대로만
기록한다.

두 방향(HWPX 파서: 필드만 갱신, HWP5 직렬화기: flags만 사용) 사이의 비대칭
때문에, HWPX 출처 문서를 HWP5로 저장하면 홀/짝 쪽번호 시작 설정이 소리 없이
유실된다(재로드 시 항상 0=이어서로 읽힘).

## 2. 재현 (red)

`src/serializer/control/tests.rs` 에 다음 라운드트립 테스트를 추가해
`flags: 0, page_num_type: 1` 인 `SectionDef` 를 HWP5 직렬화 → 재파싱하면
`page_num_type` 이 1 → 0 으로 붕괴함을 확인했다.

```
test serializer::control::tests::test_roundtrip_section_def_page_num_type_without_flags_sync ... FAILED
  left: 0
 right: 1
```

## 3. 수정

`serialize_section_def` 에서 `flags` 를 쓰기 직전 `page_num_type` 값으로
bit 20-21 을 재구성한다(HWP5 표현의 유일한 소스인 flags 필드에 최종 반영):

```rust
let flags = (sd.flags & !0x0030_0000) | (((sd.page_num_type as u32) & 0x03) << 20);
w.write_u32(flags).unwrap();
```

동일 패턴이 이미 편집 경로인
`document_core/queries/rendering.rs::apply_section_def_json` 에 존재해,
직렬화 경로에도 동일 계약을 맞췄다.

## 4. 주요 변경

- `src/serializer/control.rs`
  - `serialize_section_def`: flags 기록 직전 page_num_type→bit 20-21 재구성
- `src/serializer/control/tests.rs`
  - `test_roundtrip_section_def_page_num_type_without_flags_sync` 추가 (red→green)

## 5. 검증 결과

통과:

- `cargo fmt --check` (수정 파일 기준, 기존 CRLF 노이즈 제외)
- `RUSTFLAGS="-C linker=rust-lld" cargo clippy --lib -- -D warnings`
- `RUSTFLAGS="-C linker=rust-lld" cargo test --lib serializer::control`
  - 21 passed (신규 테스트 포함)
- `RUSTFLAGS="-C linker=rust-lld" cargo test --lib`
  - 2553 passed, 1 failed(무관), 7 ignored
  - 실패 1건(`renderer::font_paths::tests::env_font_paths_parses_and_filters`)은
    본 변경과 무관한 기존 Windows 환경 이슈(`/tmp` 경로 파싱)로, worktree
    생성 직후 clean devel 기준으로도 동일하게 재현되어 pre-existing으로 판단.

## 6. 리스크

- flags 비트 20-21 외 다른 비트에는 영향 없음(마스크 `0x0030_0000`으로 해당
  비트만 재구성).
- HWP5 원본에서 파싱된 문서는 `page_num_type` 이 이미 `flags`에서 파생된
  값이므로 이 변경으로 동작이 바뀌지 않는다(round-trip 항등).

## 7. 결론

Task M100-3195 구현과 검증을 완료했다. PR 생성 후 이슈를 close할 수 있다.
