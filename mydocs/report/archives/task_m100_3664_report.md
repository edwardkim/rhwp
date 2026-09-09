---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3664.md
last_verified: 2026-08-03
---

# Task #3664 최종 보고 — Native FFI 크레이트 복구와 재발 방지 가드

- Issue: [#3664](https://github.com/edwardkim/rhwp/issues/3664) (M100) —
  [#3202](https://github.com/edwardkim/rhwp/issues/3202) 분리
- 브랜치 `local/task3664` / 2026-08-03 당일 완결
- 단계 기록: `mydocs/working/task_m100_3664_stage{1,2,3}.md`

## 결과

| 검사 | 착수 전 | 완료 후 |
|---|---|---|
| 빌드 | **실행 불가**(워크스페이스 충돌) | **성공** 1.95초 |
| 컴파일 오류 | **2건** | 0 |
| `cargo fmt --check` | **위반 1건** | 통과 |
| `cargo clippy -- -D warnings` | **오류 1건** | 통과 |
| CI 가드 | **0건** | Lint job 에 build + clippy |

## 근인과 수정

코어의 `get_control_image_{data,mime}_native` 가 `cell_path: &[(usize, usize, usize)]`
를 받도록 바뀌었는데(#1161) FFI 가 3인자로 불러 컴파일이 깨져 있었다. 본문 문단을
뜻하는 `&[]` 를 넘긴다 — `src/main.rs:5340` CLI 경로와 같은 처리이며, 상수
`BODY_PARA` 로 이름을 주고 "FFI 표면은 셀/글상자 안 컨트롤을 아직 지정할 수 없다"는
이유를 주석에 남겼다.

## 예상 밖의 발견 — 위생 검사도 통째로 빠져 있었다

계획서는 "컴파일 오류 2건"만 잡았으나, 워크스페이스에 넣자 **한 번도 돌지 않았던
fmt·clippy 가 걸렸다.**

- fmt 위반 1건
- clippy 오류 1건 — `rhwp_string_free` 가 raw 포인터를 역참조하면서 `unsafe` 미표시

`pub unsafe extern "C"` 로 바꾸고 `# Safety` 문서화 주석을 달았다. **C ABI 심볼은
불변**이라 C# 래퍼(`DllImport` + `extern void rhwp_string_free(IntPtr)`)는 그대로
동작한다(실측 확인). 작업지시자 A안 승인으로 이번 범위에 포함했다.

이 발견이 이슈의 명제를 넓힌다 — **"가드 밖에 있으면 썩는다"가 컴파일뿐 아니라
위생 축에서도 성립했다.** 가드가 `-D warnings` 를 포함해야 하는 근거이기도 하다.

## 재발 방지 가드 — 이 작업의 본질

`①` 만 고치면 다음 API 변경 때 똑같이 깨진다. Lint job 에 가드를 넣었다:

```yaml
- name: Check Native FFI bindings
  run: |
    cargo build -p rhwp-native-ffi
    cargo clippy -p rhwp-native-ffi --all-targets -- -D warnings
```

### `-p` 지목의 실측 근거

워크스페이스 편입만으로는 부족했다.

| 명령 | FFI 검사 |
|---|---|
| `cargo clippy`(현행 CI) | **안 봄** — 루트 크레이트만 |
| `cargo clippy --workspace` | 봄. 그러나 `rhwp-subsecond` 의 기존 실패를 함께 끌어옴 |
| **`-p rhwp-native-ffi`** | **봄, 부작용 없음** ← 채택 |

### red-check — 두 축 검출력 증명

| 되돌린 결함 | 결과 |
|---|---|
| `cell_path` 인자 제거(원래 결함 재현) | 오류 **3건 검출** |
| `unsafe` 표시 제거 | clippy 오류 **3건 검출** |

가드가 "통과만 하는 것"이 아니라 실제로 잡는다.

## 검증

release-test 전체 exit 0 · clippy `-D warnings` exit 0 · fmt 통과 · FFI 단독
build/clippy 통과 · 워크플로 YAML 통과.

`Cargo.lock` 변경은 **7줄**(크레이트 등록만, 외부 의존 0 추가). 캐시 키가 바뀌어
rust-cache 새 세대가 1회 생기지만 어제 도입한 #3684 스윕이 정리한다.

## 이슈 기술 1건 정정

이슈 본문의 **"C#·Swift 래퍼 동반 사망"** 은 표현을 좁혀야 한다. 두 래퍼가 이 함수를
직접 참조하는 코드는 **0건**(grep 실측)이다. 정확히는 *"크레이트 전체가 빌드되지
않으므로 그 위에 얹힌 래퍼도 쓸 수 없다"* 는 뜻이며, 개별 함수 의존이 아니다.
내가 등록한 이슈이지만 실측으로 정정한다.

## 남긴 것

- **`rhwp-subsecond` 의 `--workspace` 실패** — feature 조건부라 기본 빌드에서 깨진다.
  이 작업과 무관한 기존 성질이나, 언젠가 `--workspace` 로 넓히려면 먼저 풀어야 한다.
- **C#·Swift 래퍼 자체 검증** — 크레이트가 살아났으므로 그 위에서 별도 확인 필요.

## 교훈

**"컴파일이 깨졌다"는 증상이었고 병은 가드 부재였다.** 워크스페이스에 넣는 순간
컴파일 2건 외에 fmt·clippy 위생 문제가 함께 드러난 것이 그 증거다. 가드 밖의 코드는
한 종류가 아니라 **모든 종류의 검사에서 동시에 빠진다.**
