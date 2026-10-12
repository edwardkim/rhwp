# Gym 시험지 입력 과제·기준풀이·자동 코퍼스 선택 제외

Issue: #7445 완료 후보의 사용자 요청. 제품 코드 head는 `7589a9323`이며, 작업 브랜치는 `fix/7445-completed-20261009`다. 사용자가 지정한 시험지 원본은 보존하고 이를 참조하는 Gym 검사를 제거했다.

## 제거 범위

| pack | 제거 과제 수 | ID |
| --- | ---: | --- |
| casual-rides | 4 | CR16, CR17, CR18, CR40 |
| core-cli | 2 | T20, T54 |
| corpus-diagnostics | 15 | CD17, CD18, CD21, CD22, CD25, CD26, CD27, CD28, CD29, CD30, CD33, CD41, CD44, CD45, CD46 |
| extraction | 12 | EX04, EX08, EX09, EX10, EX11, EX12, EX13, EX14, EX25, EX26, EX27, EX28 |
| layout-rendering | 13 | LR11, LR12, LR13, LR16, LR19, LR23, LR24, LR31, LR34, LR39, LR40, LR42, LR46 |
| objects-media | 4 | OM25, OM29, OM38, OM39 |
| oracle-probe | 2 | OP35, OP36 |
| render-tree | 11 | RT04, RT23, RT24, RT25, RT26, RT27, RT28, RT29, RT32, RT38, RT39 |
| security | 4 | SE23, SE34, SE55, SE68 |
| serialization | 12 | SR10, SR14, SR18, SR28, SR29, SR30, SR31, SR33, SR34, SR38, SR46, SR54 |
| text-editing | 7 | TE29, TE36, TE37, TE38, TE39, TE76, TE82 |

과제 JSON 86개와 같은 ID의 짝 기준풀이 JSON 86개를 제거했다. 원본 HWP/HWPX/HML 60개 tracked 파일의 SHA-256은 제거 전후 모두 같다. 과제의 지시·check·짝 기준풀이에서 지정 문서를 참조하는 활성 항목은 0개다.

Gym의 자동 코퍼스 손상 검사도 원본 시험지를 다시 선택하지 않도록 `gym/core/input_selection.py`의 54개 고유 파일명으로 제외한다. 기존 `fuzz_corpus.py`·`robustness.py`의 선택 경로에 적용하며 원본 바이트를 읽거나 삭제하지 않는다. 동일 파일명의 복사본과 NFC/case 차이도 처리한다. 새 과제나 새 테스트 함수는 추가하지 않았다. 기존 선택 시험은 임시 합성 파일명 목록으로 제외를 검사하며 실제 시험지 원본을 입력하지 않는다.

## 구조 계약 보완

- 정상 과제의 ID와 task/reference 짝 검사는 유지하고 명시적으로 제거한 ID만 제외했다. ID를 재번호화하거나 재사용하지 않았다.
- 시험지만 검사하던 render-tree 후속 쪽 선택 시험 한 함수를 제거했다. 혼합 과제 계열은 남은 정상 과제를 계속 검사한다.
- 현재 목록은 21 pack / 949 task와 reference / 최대 2,194점이다. 현재 README와 집계를 갱신했으며 2026-09-02 같은 날짜가 붙은 과거 분류 문서는 당시 기록으로 보존한다.
- 남은 과제의 채점 연산자·expect_exits·정답 권위 분류를 바꾸지 않았다. Gym 결과를 제품 렌더링 정확성 또는 PR 승인 근거로 승격하지 않는다.

## 검증

1. 수정 전 Gym 단위 검사는 macOS 기본 임시 경로의 `/var`와 `/private/var` 별칭 기대값 차이 1건이 있었다. ignored output의 실제 경로로 TMPDIR을 고정한 재실행에서는 3,171건 실행 / 실패 0 / 기존 skip 1이었다. 해당 검사나 구현을 제거해 숨기지 않았다.
2. 최종 Gym 단위 검사: 3,170건 실행 / 3,169 PASS / 0 FAIL / 기존 skip 1, 10.866초, exit 0. 삭제한 전용 함수 한 개 때문에 실행 수가 하나 감소했다.
3. `audit.py --json`: 21 pack / 949 task, issue 0, ok true. 고아 기준풀이·짝 누락이 없다.
4. `oracle_probe.py --json` 및 `--selftest --json`: 둘 다 ok true, issue 0.
5. 지정 원본 60개 해시 보존, 활성 시험지 과제 0, `git diff --check`를 확인했다. 비교 TSV·로그·과정 JSON은 ignored output에만 보존한다.

이는 Gym 구조 계약 검증이며 전건 양성·음성·경로 벤치마크 완주를 주장하지 않는다. 사용자 지시의 전체 Rust nextest는 별도로 `output/pr-review/7445-completed-20261009/stage426-full-nextest/nextest.log`에서 실행 중이다. Rust source/test는 `7589a9323`으로 고정했으며 그 실행 중에는 수정하지 않는다. 실패별 독립 근거와 관계형 기대값 검토는 완료 결과를 받은 뒤 진행한다.
