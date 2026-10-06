---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3664.md
last_verified: 2026-08-03
---

# Task #3664 Stage 1 보고 — 크레이트 복구

## 결과

`bindings/Native`(`rhwp-native-ffi`)가 **빌드·fmt·clippy 전부 통과**한다.

| 검사 | 착수 전 | 완료 후 |
|---|---|---|
| 빌드 | 워크스페이스 충돌로 **실행 불가** | **성공** (1.95초) |
| 컴파일 오류 | 2건 (`lib.rs:216`,`217`) | 0 |
| `cargo fmt --check` | **위반 1건** | 통과 |
| `cargo clippy -- -D warnings` | **오류 1건** | 통과 |

## 수정 내용

### ① 컴파일 오류 — `cell_path` 인자 누락

코어의 `get_control_image_{data,mime}_native` 가 `cell_path: &[(usize, usize, usize)]`
를 받도록 바뀌었는데(#1161) FFI 는 3인자로 불렀다. 본문 문단을 뜻하는 `&[]` 를 넘긴다
— `src/main.rs:5340` CLI 경로와 같은 처리다. 상수 `BODY_PARA` 로 이름을 주고 "FFI
표면은 셀/글상자 안 컨트롤을 아직 지정할 수 없다"는 이유를 주석에 남겼다.

### ② 워크스페이스 편입

루트 `members` 에 `bindings/Native` 추가. 의존이 `rhwp_core` 하나뿐이라 새 외부
크레이트는 늘지 않는다.

### ③ 위생 정리 (계획 범위 밖 — 작업지시자 A안 승인)

워크스페이스에 넣자 **그동안 한 번도 돌지 않았던 검사가 걸렸다.**

- **fmt 위반 1건** — `cargo fmt -p` 로 정리(6+/9−).
- **clippy 오류 1건** — `rhwp_string_free` 가 raw 포인터를 역참조하면서 `unsafe` 로
  표시되지 않았다. `pub unsafe extern "C"` 로 바꾸고 `# Safety` 문서화 주석을 달았다.

**C ABI 심볼은 바뀌지 않는다** — C# 래퍼(`DllImport` + `private static extern void
rhwp_string_free(IntPtr)`)는 심볼만 참조하므로 호출 측 변경이 없다. 실측 확인했다.

## 이 발견의 의미

계획서는 "컴파일 오류 2건"만 잡았으나, 실제로는 **위생 검사도 통째로 빠져 있었다.**
이 이슈가 드러내려던 "가드 밖에 있으면 썩는다"가 컴파일뿐 아니라 fmt·clippy 축에서도
성립한 것이다. Stage 2 의 CI 가드가 `-D warnings` 를 포함해야 하는 근거이기도 하다.

## 다음

Stage 2 — CI 가드 추가 + red-check(가드 제거 시 실패 증명).
