# #3587 Stage 9 — C1 내용 채우기 선검증

- 승인: 2026-09-12 메인테이너 「다음 절차 진행을 승인합니다」 — C 상세계획 승인.
- 선행: [B 결과 승인](task_m100_3587_stage8.md), [C 상세계획](../plans/task_m100_3587_impl_c.md).
- 시작 HEAD: `048dbb072`, 브랜치 `task_m100_3587`.
- 상태: 선검증 절편 구현·집중 검증 완료. **C1 전체 완료나 저장·시각 검증 완료가 아니다.**

## 이번 변경

C1의 첫 연결점인 읽기 전용 `validate_template_fill_native`를 구현한다.
원형 상대 문단 경로와 텍스트 범위/같은 문단의 닫힌 필드를 지정하고 모든 record를 먼저 검사한다.
B 소유 트리 탐색·참조 폐쇄성·구조 예산을 재사용하며 문서 전체 복제나 setter 호출은 하지 않는다.
목적지 삽입 경계는 B 계약으로 검증하되 생성된 ID/경로를 이미 존재하는 결과처럼 반환하지 않는다.

- 누락/여분/중복 key와 빈 문자열을 구분한다.
- Unicode scalar 범위, 중첩 글상자/셀 경로, 필드 경계, 컨트롤 anchor, 중복/겹친 대상을 검사한다.
- record 수와 복사 수, 10,000개 확장 target, 경로 깊이, 입력 text byte 예산을 검사한다.
- 길이가 긴 한 문단을 많은 target이 반복 검사하는 비용도 제한한다.
- UTF-8 대체문자열 바이트와 B 구조 예산을 함께 확인한다. 이후 실제 staging에서 늘어나는
  runs/offsets/line 자료의 비용까지 검사한 것으로 보고하지 않는다.
- multi-paragraph field, field 안의 내부 control slot, 정확한 anchor 근거 없는 target은 거부한다.
  이런 원형의 B 복사 지원을 제거하는 변경은 아니다.

## 검증 계획

새 `tests/cases/issue_3587_template_fill_preflight.rs`에서 정상/오류 모두 IR·이벤트·clipboard
무변경을 검사한다. 실물 `samples/rnote/labnote-001.hwp`의 pi=12 내부 셀도 직접 호출한다.
기존 #3587 계약과 함께 실행하며 필수 Rust lint는 기존 review worktree와 고정 target을 사용한다.
생성 harness는 review 전용이고 source PR에 포함하지 않는다. 전체 회귀·Docker WASM·시각 비교는
C 종료 게이트에서 별도로 진행하며 이번 읽기 전용 API의 통과로 대체하지 않는다.

## 현재 미완료

- 실제 fixed-form/복사본 채우기, 변경 대상 staging과 단일 반영
- 이름 기반의 블록 내부 필드 편의 선택, 셀 행/열 편의 주소
- 값 변경 뒤 파생 메모리 예산·reflow와 HWP/HWPX 저장 재열기
- C2 행 복제, C3 public adapter, D, Gym

#7065의 Studio 재편집 페이지네이션은 범위 밖이다. 새 HWP/HWPX나 시각 판정용 산출물은
이번 선검증 작업으로 생성하지 않는다. 원격 push·PR·GitHub 댓글은 진행하지 않는다.

## 실행 중 발견·정정

1. main checkout의 오래된 generated harness가 삭제된 원본을 참조하여 최초 `cargo fmt --all`이
   실패했다. main의 generated 파일을 고치지 않고 기존 review worktree에서 prepare 후 검증했다.
2. 최초 review prepare 뒤 fmt가 새 source 크기를 변경하면서 자동 suite 배정이 달라졌다.
   resolver가 선택한 suite에서는 테스트가 0건이라 nextest exit 4였다. 성공으로 기록하지 않았다.
   `fmt → prepare → 실제 case 실행` 순서로 정정한 후 신규 10건 실행을 확인했다.
3. 새 fixture의 `Control::Shape` 구성에서 `Box`가 누락된 컴파일 오류를 수정했다.
   이는 테스트 작성 오류이며 제품 결함이나 정상 샘플의 실패로 분류하지 않는다.
4. native Clippy의 `manual_slice_size_calculation` 1건을 `size_of_val`로 보정했다.
   해당 표현 보정 후 fmt 적용/check, lint 묶음과 집중 계약을 재확인한다.

보존 로그는 `output/3587/c1-preflight/`다. `first-compile.log`와
`clippy-native-first.log`는 실패 기록이며 최종 통과 로그와 분리한다.
`source-sha256.txt`의 제품/테스트 5파일은 main과 review에서 byte-identical임을 확인했다.
review 기반 HEAD와 main HEAD의 나머지 제품·테스트·Cargo 차이는 없고 문서만 다르다.

## 검증 기록

- 신규 선검증 10건 최초 정상 실행 PASS; 로그 `focused.log`.
- 기존 #3587 81건을 포함한 집중 91건 최초 실행 PASS; 로그 `contracts.log`.
- Clippy 표현 보정 후 최종 동일 코드 검증 결과는 아래에 별도로 기록한다.
- 전체 회귀, Native Skia, Docker WASM, 한컴 시각 검증은 이번 회차에서 실행하지 않는다.
  새 요청이 값을 쓰는 단계까지 완성되지 않았으므로 이 기록으로 C1 전체를 완료 처리하지 않는다.

### 최종 동일 코드 결과

| 검증 | 결과 | 로그 (`output/3587/c1-preflight/`) |
| --- | --- | --- |
| 기존 81건 + 신규 선검증 10건 | 91 PASS, 같은 suite의 비대상 1,637건 skipped | `contracts-final.log` |
| Rust fmt check | PASS | `fmt.log` |
| native Clippy (`-D warnings`) | PASS | `clippy-native.log` |
| WASM lib Clippy (`-D warnings`) | PASS | `clippy-wasm.log` |
| workspace build | PASS | `workspace-build.log` |
| workspace/all-target Clippy (`-D warnings`) | PASS | `clippy-workspace.log` |
| integration manifest check | PASS | `manifest.log` |

변경 문서 5개 링크 검사와 `git diff --check`도 통과했다. nextest 설치 버전 0.9.137이
저장소 권장 0.9.140보다 낮다는 경고는 남았으며, 테스트 실행 결과는 exit 0이다.
생성 suite·manifest·로그는 커밋 대상이 아니다.

다음 절차는 승인된 C1 안에서 실제 복사본에 값을 채우는 staging과 단일 반영을 구현하고,
마지막 record 실패 시 원본·clipboard·이벤트가 그대로인지 검증하는 것이다.
이번 통과는 요청의 읽기 전용 검사에 대한 증적이며 실제 채우기 성공이나 rollback 구현의
증적을 대신하지 않는다. 원격 push·PR·Gym 실행은 하지 않았다.
