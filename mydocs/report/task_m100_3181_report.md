---
kind: report
status: done
last_verified: 2026-07-23
---

# Task #3181 처리 결과 — CFB LenientCfbReader DIFAT 순환 감지 누락

Issue: #3181

## 문제

`src/parser/cfb_reader.rs` 의 `LenientCfbReader::open` 이 CFB 헤더의 DIFAT 확장 섹터
체인을 순회할 때 FAT/미니FAT 체인 순회(`read_chain_static`)와 달리 방문 섹터 집합을
추적하지 않았다. 순회 상한은 파일 헤더에서 그대로 읽은 `difat_sectors_count`
(`u32`, 공격자 통제 가능, 최대 4,294,967,295)뿐이라, 두 DIFAT 섹터가 서로를
가리키는 순환을 만들고 `difat_sectors_count` 를 `u32::MAX` 로 설정하면 실제 체인
길이(2)와 무관하게 최대 42억 회 넘게 동일한 섹터를 반복 순회한다. 패닉은 아니지만
단일 스레드 WASM 환경(브라우저 탭)에서는 사실상 응답 없음(DoS)이 된다.

## 재현

`src/parser/cfb_reader.rs::tests::lenient_open_terminates_on_cyclic_difat_chain`
테스트로 재현. 헤더에 서로를 가리키는 DIFAT 섹터 2개를 배치하고
`difat_sectors_count = u32::MAX` 로 설정한 뒤, `LenientCfbReader::open` 호출을
별도 스레드에서 실행해 3초 타임아웃 내 반환하는지 확인한다.

- 수정 전: 3초 타임아웃 내 반환하지 못해 테스트 실패(RED, `cargo test` 로 확인).
- 수정 후: 방문 집합에 의해 즉시(0.06초 이내) 순환을 감지해 종료(GREEN).

## 수정

DIFAT 확장 섹터 순회 루프에 `HashSet<u32>` 방문 집합을 추가했다. 이미 방문한
섹터 ID 로 돌아오면 즉시 순회를 중단한다. FAT/미니FAT 체인 순회에 이미 있던
동일한 패턴(`read_chain_static`)을 DIFAT 순회에도 적용한 것이라 별도 개념 도입이
아니다.

변경 파일: `src/parser/cfb_reader.rs` (DIFAT 확장 섹터 순회 루프, 순회 로직
약 6줄 추가) + 회귀 테스트 1건 추가.

## 검증

```
RUSTFLAGS="-C linker=rust-lld" cargo test --lib parser::cfb_reader
# test result: ok. 9 passed; 0 failed

RUSTFLAGS="-C linker=rust-lld" cargo test --lib parser::
# test result: ok. 427 passed; 0 failed
```

기존 CFB 관련 테스트(헤더 필드 검증, 손상된 디렉터리 엔트리 name_len 등)와
전체 `parser` 모듈 테스트 모두 회귀 없이 통과.

## 영향 범위

`LenientCfbReader::open` 은 `parser/mod.rs` 에서 표준 `cfb` 크레이트 파서가
실패한 뒤 fallback 으로만 호출되므로, 이번 수정은 이미 손상이 확인된 입력
경로에만 영향을 준다. 정상 CFB 파일의 파싱 결과에는 영향이 없다.
