# task-m100-1: hatchStyle 직렬화 catch-all 수정

## 문제

`src/serializer/hwpx/shape.rs`의 `hatch_style_str`은 `parse_hatch_style`
(파서, `src/parser/hwpx/utils.rs`)의 역매핑이다. 파서 쪽은 `HORIZONTAL`~
`CROSS_DIAGONAL` 6개 값을 전부 명시적으로 나열하는데, 직렬화 쪽은 마지막
값(6, `CROSS_DIAGONAL`)만 `_` catch-all에 얹혀 있었다.

```rust
fn hatch_style_str(pattern_type: i32) -> &'static str {
    match pattern_type {
        1 => "HORIZONTAL",
        2 => "VERTICAL",
        3 => "BACK_SLASH",
        4 => "SLASH",
        5 => "CROSS",
        _ => "CROSS_DIAGONAL",
    }
}
```

`pattern_type`은 HWP5 binary(`doc_info.rs`)와 HWP3(`drawing.rs`)에서
원시 정수로 읽혀 IR에 들어오므로, 계약(1~6) 밖의 값(손상 파일, 0, 7 이상)이
들어와도 이 함수는 조용히 `CROSS_DIAGONAL`을 방출한다. 즉 계약 밖 값을
정당한 무늬로 둔갑시켜 라운드트립 시 원본과 다른 무늬가 저장될 수 있다.
같은 파일의 `lineShape` style 매핑(줄 674~676 주석)에서 이미 한 번 겪은
"catch-all이 유효한 값처럼 보이는" 패턴(#1531)과 동일한 종류다.

## 수정

6을 명시적으로 나열하고, catch-all은 6개 계약 값 중 가장 무난한
`HORIZONTAL`로 바꿨다(무늬 정보가 없다는 신호를 유지하되 임의의 특정
무늬로 확정 짓지 않기 위함).

## 테스트

`task_m100_hatch_style_str_covers_all_six` — 1~6 및 계약 밖 값(99)의
매핑을 검증.

## 검증

이 PC의 Rust 툴체인이 `dbghelp.lib` 손상(`CVT1107`/`LNK1123`)으로 모든
빌드 스크립트 링크에 실패하는 상태라 `cargo check --lib` 실행이 불가능했다.
무관한 crate(`proc-macro2`, `paste`, `serde_core` 등)의 빌드 스크립트도
동일하게 실패하므로 이 변경과 무관한 환경 문제로 판단한다. 변경 자체는
`match` 분기 추가 및 단위 테스트 추가뿐이라 문법·타입 위험이 낮다.
