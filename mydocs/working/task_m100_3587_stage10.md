# #3587 Stage 10 — C1 복사본 채우기·단일 반영

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」. 기록일: 2026-09-13 KST.
- 기준: `f9e20e016`, `task_m100_3587`.
- 계획: [C 상세계획](../plans/task_m100_3587_impl_c.md), [선검증 결과](task_m100_3587_stage9.md).
- 상태: 복사본별 native 채우기·단일 삽입 구현 및 집중 검증 완료. C1 전체·C2/C3·D/Gym 완료가 아니다.

기존 B 복사 staging에 모델 변경 callback을 연결하고 모든 복사본의 값 채우기가 성공한 뒤
기존 삽입 지점으로 한 번 반영한다. callback은 살아 있는 DocumentCore에 접근하지 않는다.
기존 필드 setter의 모델 편집을 공유하며 텍스트 범위는 뒤에서 앞으로 변경한다.
본문/중첩 셀/글상자/캡션 소유 경로를 사용하며 수정된 셀의 조상 표에만 재계산 provenance를 전달한다.

검증 대상은 복사본별 다른 값, 원형·주변·빈 문단·clipboard 보존, 입력 오류/마지막 복사본
staging 예산 오류 시 IR·이벤트 무변경, 한글/보조 평면 문자/줄바꿈, 필드 상태·저장 재열기다.
새 sample과 baseline, Studio 페이지네이션 #7065는 변경하지 않는다.
고정 양식 채우기·편의 주소·public adapter·Gym 및 전체 종료 검증은 아직 남아 있다.

## 구현 경계

- `repeat_paragraph_block_prepared`: B의 준비·삽입 흐름을 공유한다. 기존 복사 API는
  내용을 바꾸지 않는 callback을 사용하므로 기존 호출 방식은 유지한다.
- `repeat_and_fill_paragraph_block_native`: 모든 record와 원형 상대 경로를 먼저 검사하고,
  새 ID가 부여된 복사본에 각 record의 값을 넣는다. 뒤쪽 범위부터 처리해 앞쪽 치환으로
  다른 대상의 위치가 밀리지 않게 한다. 결과는 기존 B의 복사본별 경로 대응표다.
- `replace_field_text_model`: 기존 필드 setter에서 모델 편집만 추출했다. 글자 서식·필드 범위·
  UTF-16 offset·채워진 필드 상태(bit 15) 갱신을 같은 코드로 처리한다.
- 입력 값뿐 아니라 기존 문단을 문자 배열로 펼치는 작업량도 제한한다. 복사본별 변경 후
  실제 소유 구조 비용을 누적 검사한다. 실패하면 준비한 복사본을 버리고 live 문서에는 삽입하지 않는다.
  process abort/OOM 복구나 전체 process RSS 상한을 보증하는 계약은 아니다.
- 표 내용 변경은 대상 셀을 소유하는 조상 표의 재계산 표시를 전달한다. 표 크기 고정이나
  줄간격 축소, 빈 문단 삭제로 결과를 맞추지 않는다. Studio의 #7065 수정으로 보고하지 않는다.

## 검증 진행 기록

- 첫 집중 실행 6건 PASS (`output/3587/c1-fill/focused.log`).
  연구노트 내부 셀에 복사본별 다른 값이 들어가고 HWP/HWPX 메모리 저장·재열기에서 유지됨을 확인했다.
- 이후 중첩 글상자·두 번째 복사본 staging 실패 계약과 기존 텍스트 작업 예산을 보강했다.
  위 첫 통과를 보강 후 최종 결과로 재사용하지 않는다. 최종 결과는 아래에 별도 기록한다.
- 새 파일을 디스크에 내보내거나 한컴으로 열어 본 결과는 아니다. 저장 재열기 계약을
  메인테이너 시각 통과로 표현하지 않는다.

### 음성 대조와 서식 경계 보정

1. 기존 #3587 계약과 새 채우기 8건의 첫 결합 실행은 99 PASS였다 (`contracts.log`).
2. review worktree에서만 값 채우기 callback을 생략한 음성 대조는 8건 중 6 FAIL/2 PASS,
   exit 100이었다 (`negative-no-fill.log`). 본문/필드/글상자/실물 저장과 중간 staging 실패 검사가
   실제 값 채우기 누락을 검출했다. 검증용 변경은 복원했으며 작업 브랜치에는 적용하지 않았다.
3. `6667bd68a` 검증 도중 범위 삭제 후 오른쪽 서식이 상속되는 경계를 코드 점검에서 발견했다.
   이는 일반 삭제의 기존 보호 규칙이지만 템플릿 값은 원래 대상의 서식을 상속해야 한다.
   불필요한 구형 코드 lint를 계속하지 않도록 해당 검증 묶음을 중단했다(exit 143,
   `contracts-before-style-interrupted.log`). 중단 결과를 PASS로 기록하지 않는다.
4. 대상 스타일 0/오른쪽 스타일 1을 둔 신규 계약은 보정 전 1 FAIL(exit 100)이었다
   (`style-red.log`, 실제 `Some(1)`/기대 `Some(0)`). 템플릿 경로에서만 대상 시작점 서식을
   기억하고 값 삽입 후 그 범위에 적용한다. 기존 삭제와 필드 setter의 기존 호출 동작은 변경하지 않는다.

## 최종 검증 결과

제품·테스트 커밋은 `9f96b4b2e`이며 기존 review worktree를 해당 커밋으로 전환한 뒤 검증했다.
아래 로그는 `output/3587/c1-fill/`에 보존한다. 이후 계획/결과 기록은 문서만 변경한다.

| 검사 | 최종 결과 | 로그 |
| --- | --- | --- |
| #3587 집중 계약 (기존 91 + 신규 채우기 9) | 100 PASS, 비대상 2,011 skipped | `contracts-final.log` |
| 기존 `field_query::` 검사 | rhwp 14 PASS, 다른 workspace member는 필터로 0건 | `field-legacy.log` |
| 전체 fmt check | PASS | `fmt.log` |
| native Clippy `-D warnings` | PASS | `clippy-native.log` |
| WASM lib Clippy `-D warnings` | PASS | `clippy-wasm.log` |
| workspace build | PASS | `workspace-build.log` |
| workspace/all-target Clippy `-D warnings` | PASS | `clippy-workspace.log` |
| integration manifest check | PASS, 1,275 source / 48 target | `manifest.log` |

실행은 Cargo 명령을 병렬로 겹치지 않고 고정 target `/home/edward/mygithub/rhwp/target/pr-review`에서
순차 수행했다. 집중 계약은 현재 manifest의 `resolveCase`로 `issue_3587_*` 12개 원본의 suite를
해석하고 `cargo nextest run --locked --test <각 suite> -E 'test(issue_3587_)'
--cargo-profile release-test --target-dir <고정 target>`으로 실행했다.
기존 필드 검사는 `cargo test --locked --profile release-test --lib field_query::
--target-dir <고정 target>`이다. lint 명령은 AGENTS.md의 세 Clippy와 workspace build 묶음이다.

nextest 0.9.137/권장 0.9.140 차이 및 해당 버전에서 알 수 없는 JUnit 설정 경고는 남는다.
실행 0건을 성공 증거로 삼지 않았고 실제 대상 100건·14건을 로그에서 확인했다.
변경 문서 5개의 링크 검사와 `git diff --check`도 통과했다. generated harness/manifest는
검증 전용이며 source 커밋에 포함하지 않는다.

## 다음 절차와 미검증 범위

다음은 C1의 고정 양식 채우기와 블록 범위의 이름/셀 좌표 편의 선택이다.
그 뒤 C2 행 복제, C3 WASM·CLI·MCP 연결 및 dry-run 준비 경로를 진행한다.
`validate_template_fill_native`는 여전히 읽기 전용 사전 검사이지 staging 비용까지 통과한
dry-run 완료 결과가 아니다. C3에서 실제 준비 경로를 공유해야 한다.

이번에는 전체 nextest·Native Skia·Docker WASM·한컴 시각 검증·C 채우기 비용 계측을 실행하지 않았다.
기존 B의 전체 통과를 이번 C 코드에 재사용하지 않는다. 실제 화면 조판과 모든 자동화 흐름의
완료 판정은 C 종료 검증에 남는다. #7065, D, Gym, 원격 push·PR·이슈 close는 진행하지 않았다.
