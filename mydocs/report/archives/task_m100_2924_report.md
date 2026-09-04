# Task #2924 처리 결과 — HWP3 조합형(Johab) 무효 종성 인덱스 오처리 수정

## 이슈

- edwardkim/rhwp#2924 — HWP3 조합형(Johab) 디코더가 무효 종성 인덱스를 "받침 없음"으로 오인해
  존재하지 않는 완성형 음절을 조용히 생성함.

## 문제 분석

`src/parser/hwp3/johab.rs`의 `decode_johab`은 조합형 2바이트 코드의 종성(jong) 5비트 필드를
`jong_map` 테이블로 변환한다. 이 테이블에서 인덱스 1은 "받침 없음"(값 0)을 나타내고, 인덱스
0/18/30/31은 표준 조합형 인코딩에서 사용되지 않는 예약/무효 값으로 `-1`이 배정되어 있다.

기존 코드는 다음과 같이 `jong == -1`인 모든 경우를 예외 없이 "받침 없음"(0)으로 치환했다.

```rust
let mut jong = jong_map[jong_idx as usize];

if cho != -1 && jung != -1 {
    if jong == -1 {
        jong = 0;
    }
    let uni_val = 0xAC00 + (cho * 21 * 28) + (jung * 28) + jong;
    ...
}
```

이는 "종성 필드가 없음을 뜻하는 유효한 상태"와 "종성 필드 자체가 예약/무효 값인 상태"를 구분하지
못해, `jong_idx`가 0/18/30/31인 예약/무효 조합조차 정상적인 완성형 한글 음절로 합성해 반환하는
결과를 낳는다. 이는 `src/parser/hwp3/encoding.rs`의 `decode_hwp3_string`이 짝이 없는 후행 바이트
같은 무효 입력에 대해 명시적으로 `'?'`를 반환하는 것과 원칙이 어긋난다 — 무효 입력이 플레이스홀더가
아니라 겉보기에 정상적인, 그러나 원본에는 없던 한글 음절로 둔갑해 버린다.

## Red → Green

### Red (수정 전)

`cho_idx=2`(초성 'ㄱ'), `jung_idx=3`(중성 'ㅏ'), `jong_idx=0`(예약/무효)으로 구성한
`ch = 0x8000 | (2 << 10) | (3 << 5) | 0 = 0x8860`에 대해:

- 수정 전: `decode_johab(0x8860)` → `'가'` (U+AC00) 반환. 예약/무효 조합인데도 정상 음절처럼 보임.
- 테스트 `decode_johab_rejects_reserved_jong_index`는 `assert_ne!(decode_johab(ch), '가')`를
  검증하며, 수정 전 코드에서는 이 assert가 실패한다(수정 전 코드로 되돌려 직접 확인함).

### Green (수정 후)

```rust
let jong = jong_map[jong_idx as usize];

if cho != -1 && jung != -1 && jong != -1 {
    let uni_val = 0xAC00 + (cho * 21 * 28) + (jung * 28) + jong;
    ...
}
```

`jong != -1` 조건을 추가해 종성 필드 자체가 유효한 값일 때만 완성형 음절을 합성하도록 강화했다.
`jong_idx=1`(받침 없음, jong=0)은 여전히 정상 처리되고, 예약/무효 조합(`jong_idx` 0/18/30/31)은
합성 경로를 건너뛰어 기존 한자/기호 이진 탐색 → 최종 `'?'` fallback 경로로 넘어간다.

```
$ cargo test --lib johab
running 1 test
test parser::hwp3::johab::tests::decode_johab_rejects_reserved_jong_index ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2519 filtered out
```

## 변경 파일

- `src/parser/hwp3/johab.rs` — `decode_johab` 종성 처리 조건 수정(핵심 diff 6줄) + 회귀 테스트 1건 추가.

## 검증

- `cargo check --lib` 통과.
- `cargo test --lib johab` 통과(신규 테스트 1건 green).
- `rustfmt --edition 2021 src/parser/hwp3/johab.rs` 적용.

## 영향 범위

`decode_johab` 내부 로직 변경만으로 해결되며, 렌더러·문서 IR·다른 파서 모듈에는 영향 없음. 변경
범위가 작아 회귀 위험 낮음.
