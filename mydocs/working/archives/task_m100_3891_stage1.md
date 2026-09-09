---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3891.md
last_verified: 2026-08-03
---

# Task #3891 Stage 1·2 보고 — C 표면 계약 가드와 표류 정정

## 결과

`tests/ffi_c_surface_contract.rs` 신설 — **외부 툴체인 없이** `cargo test` 만으로
C ABI 표면과 소비자 선언의 정합을 검사한다.

| 검사 | 결과 |
|---|---|
| `c_header_matches_rust_ffi_surface` | ok |
| `csharp_wrapper_matches_rust_ffi_surface` | ok (정정 후) |

## 검출한 실제 표류 — C# 래퍼의 `rhwp_read_text` 누락

가드를 구현하자마자 **실제 불일치를 잡았다.**

```
C# 래퍼: `rhwp_read_text` 선언 없음 (Rust 는 export 함)
```

### 의도적 제외인지 판단 (계획서 §6)

시간순 실측으로 **표류임을 확정**했다.

| 시점 | 사건 |
|---|---|
| 2026-05-04 | C# 래퍼 도입(`dcd33bd52`) — 당시 `rhwp_read_text` 없음 |
| 이후 | Rust 에 `rhwp_read_text` 추가 |
| 2026-05-13 | Swift 바인딩이 그 함수를 반영(`75d76f854`, `readText` 공개 API) |

C# 만 갱신되지 않았다. Swift 는 공개 API 3개(`readText`·`exportText`·`exportMarkdown`),
C# 은 2개였다. **의도적 미지원이 아니므로 예외 등재가 아니라 정정이 답이다.**

→ `ReadText` 공개 메서드 + `[DllImport]` 선언을 추가했다. 주석에 이 경위를 남겼다.

## 설계 — 왜 인자 "개수"만 비교하나

타입 문자열을 그대로 비교하지 않는다. 같은 것을 세 표기로 쓴다:

| Rust | C | C# |
|---|---|---|
| `*const c_char` | `const char *` | `byte[]` |
| `*mut c_char` | `char *` | `IntPtr` |

공백·별표 위치까지 맞추려 들면 오탐이 난다. **이름과 인자 개수**가 어긋나는 것이
실제로 터지는 표류다.

### 실측으로 잡은 오탐 하나

첫 구현이 "Rust 4 vs C 헤더 3"을 보고했는데, **Rust 의 후행 콤마**(`page: i32,\n)`)를
인자로 세고 있었다. 콤마로 끊은 뒤 빈 조각을 버리도록 고쳐 표기 차이를 흡수했다.
**가드 자체의 오탐을 먼저 잡지 않으면 진짜 결함이 묻힌다.**

## red-check — 3축 전부 검출

| 흔든 지점 | 결과 |
|---|---|
| C# 선언 제거 | 2건 실패 |
| C 헤더에서 함수 제거 | `C 헤더: rhwp_read_text 선언 없음` |
| **Rust 인자 추가**(권위 변경) | 헤더·C# **양쪽 동시 검출** |

세 번째가 중요하다 — **권위가 바뀌면 두 소비자가 함께 잡힌다.** 이것이 이 가드의
본래 목적(#3664 에서 `unsafe` 를 바꿨을 때 래퍼 영향을 코드 판독으로만 확인했던 한계
해소)이다.

## 파싱 실패는 조용히 통과하지 않는다

세 파서 모두 결과가 비면 `assert!` 로 실패한다. 선언 형식이 바뀌어 가드가 아무것도
못 읽으면 **통과가 아니라 실패**다 — #3664 형(가드가 죽은 채 초록) 함정 차단.

## 한계 명시

선언이 일치해도 **런타임 동작·ABI 호환은 보장하지 않는다.** 실제 링크·호출 검증은
Swift·.NET 툴체인이 있는 환경에서 별도로 해야 한다. 테스트 모듈 주석과 보고서에
남겼다.
