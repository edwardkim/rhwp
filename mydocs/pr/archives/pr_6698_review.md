---
kind: pr-review
pr: 6698
reviewed_at: 2026-09-04
source_head: 7e47ef6914edfed1852c7fff99cd04cdc71713a4
---

# PR #6698 검토 - 묶음 빈칸의 전진폭

## 판정: 승인

원 PR은 `src/renderer/layout/text_measurement.rs`에서 묶음 빈칸(NBSP)의 전진폭을
일반 공백과 같게 처리한다. 원 PR head
`7e47ef6914edfed1852c7fff99cd04cdc71713a4`는 통합 후보의 `c0145ec66`으로
provenance를 보존해 적용됐다.

## 검토 범위와 결과

- 폭 계산만 변경하며 line-breaking과 글꼴 선택 계약을 넓히지 않는다.
- 회귀 테스트 `issue_6646_nbsp_advance_matches_space`가 일반 공백과 NBSP의 폭
  동치를 명시한다.
- 원 PR의 required check는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6698/checks)
- 통합 후보에서 다음 로컬 호환/통합 검증이 성공했다. 이는 공식 CI의 full lane이나
  nextest 전체 실행을 대신하지 않는다.

```sh
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 시각 증적

`samples/exam_eng.hwp`의 1쪽을 Hancom 2020 기준 PDF와 현재 `rhwp-studio`에서
직접 대조했다. 제목, 문제 영역의 2단 분할, 표와 삽화의 구조가 기준과 같은 범위로
확인됐다. 이는 NBSP 전진폭 회귀의 시각적 범위 확인이며, 문서 전체의 pixel-perfect
동치 선언은 아니다.

| 자료 | 경로 | SHA-256 |
| --- | --- | --- |
| Hancom 2020 기준 1쪽 | `../assets/pr_6683_6705_20260904/reference-6698-exam-eng-p1.png` | `eb3ab33dc71d7a60521317580a32e6221beab5c07e45b189a59f855df3f60455` |
| 기준 PDF | `samples/exam_eng-2020.pdf` | 저장소 추적 파일 |

## 메인터너 보정 판단

추가 메인터너 코드 보정은 필요하지 않다. 최종 병합 시에는 통합 PR의 최신 head CI를
다시 확인한다.
