# PR #6710 검토 - 출처 표식과 무관한 첫 조각 초과 허용치

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6710](https://github.com/edwardkim/rhwp/pull/6710) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `c340bd7a8`, `61cd71fb9` (`-x`) |
| 메인터너 보정 | `872f3d4c5` 및 정식 fixture 등록 후속 commit |
| 관련 이슈 | [#5057](https://github.com/edwardkim/rhwp/issues/5057) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head required CI는 선정 시 성공 또는 정책상 skip이었다.
- 저장된 첫 조각 source frame 허용치를 native HWP5와 direct-HWPX 저장 조판 경로 모두에
  적용하고, 기존 페이지 수 차이를 매개로 하던 `#4658` 계약도 별도 차이 입력으로 유지한다.
- source PNG는 참고 자료일 뿐 통합 head 시각 증적이 아니다.
- 기존 private-path 탐색과 export/ZIP 오류 성공 처리를 제거한다. 원본 HWP는
  `samples/issue5057/`에 SHA-256 manifest와 함께 등록하고 모든 fixture 발견 뒤 오류는 실패한다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **메인터너 보정 후 수용 가능**
- 원 head: private fixture와 export/ZIP 오류를 엄격히 검증하지 못한다.
- 보정 뒤 통합 head: 정식 fixture와 fail-closed 테스트를 포함한다.
- 수용 전 조건: 통합 Rust 검증 및 HWP5/direct-HWPX의 페이지·표 영역 직접 시각 증적.
- 원격 조치: 수행하지 않았다.
