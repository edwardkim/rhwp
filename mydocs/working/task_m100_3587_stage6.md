# #3587 Stage 6 — B2 잔여 계약·비용 계측과 B3 저장 검증

- 일자: 2026-09-12
- 승인: 메인테이너 「다음 절차를 진행하는 것을 승인합니다」.
- 기준: `377fabae6` 및 [승인된 B 계획](../plans/task_m100_3587_impl_b.md).
- 상태: 집중 계약·저장 재열기·단회 비용 계측 완료. 한컴 시각 판정 및 B 종료 통합 게이트 대기.

## 이번 절편

기존 실물 표와 연구노트 블록으로 HWP/HWPX 저장·재열기와 1/10/100회 반복 비용을 확인한다.
`table-in-tbox.hwp` pi=0의 구역 경계는 제거하지 않고 거부 결과로 별도 기록한다.
독립 HWPX가 없는 파일의 rhwp 변환본은 파서 경로 검사일 뿐 독립 정답지가 아니다.
빈 문단 0/1/2개, 명시적 쪽/단 나눔 및 사본 삭제 후 나머지 문단 보존을 추가 검사한다.

측정은 fixture/count마다 별도 프로세스에서 시행한다. 요청에는 일반(비 batch) 페이지네이션과
사전검사가 포함된다. Linux `/proc/self/status`의 RSS와 프로세스 누적 HWM을 전후 기록하며,
이 수치를 순수 복제 할당량 또는 논리 예산의 엄밀한 메모리 상한으로 해석하지 않는다.
출력 저장·SVG 생성은 시간·메모리 측정 뒤 수행한다. 한 번의 실행값은 통계적 성능 보장이 아니다.

원본과 생성 문서는 `output/3587/` 아래에서 메인테이너가 직접 열 수 있게 준비한다.
구조 계약 성공과 한컴 시각 판정은 별도로 기록한다.

## 검증 결과와 적용 범위

이번 절편의 제품 변경은 없다. 제품은 Stage 5의 `333ad5c149`이며 최종 테스트 SHA는
`698279af79`다. 동일 SHA의 detached review worktree `rhwp-review-3587`에서 suite를
준비했다. generated suite/manifest는 제출하지 않는다.

| 검사 | 결과 |
| --- | --- |
| 기존 A/B와 신규 4개 계약 | **78 PASS / 0 FAIL**, 선택 밖·ignored 합계 1,260 skipped |
| HWP 실물 2종 × 삽입 위치 4개 × 저장 형식 2개 | 16회 저장·재열기 계약 PASS |
| 파생 HWPX 입력 → 반복 → HWP/HWPX 재열기 | PASS. 독립 한컴 HWPX 정답지가 아님 |
| 빈 문단 0/1/2개 × 명시적 나눔 없음/쪽/단 | 9조합 PASS. 사본 삭제 후 다른 사본 보존 포함 |
| 그룹→글상자→표/그림, 그룹 캡션, 수식 | 소유 경로 대응·새 ID·그림 자원 참조·원형/DocInfo 불변 PASS |
| fmt / native Clippy / 신규 test target Clippy / manifest | PASS |

저장 검사는 본문/중첩 문단 텍스트·빈 문단 순서·문단/글자 스타일 참조, 표/셀 구조,
최상위 표 ID 순서, 구역 용지 정보와 바탕쪽 문단 내용을 검사한다. 모든 IR 필드의 완전 동일성이나
모든 컨트롤의 양방향 저장을 증명하지 않는다. 그룹 혼합 구조와 빈 문단 조합은 합성 API 계약이며
한컴이 정상 문서로 인정한 실물 fixture가 아니다.

### 초기 검증 오류의 원인과 정정

1. `65d9b2ed55`: 76 PASS / 1 FAIL. HWP 저장기의 기존 #1915 경로가 새 첫 문단에 구역
   정의를 보강하면서 그 안의 바탕쪽 문단까지 **본문**으로 세었던 비교 오류였다.
   `src/serializer/body_text.rs`의 `first_para_with_secd`를 확인한 후, 구역 소유 내용은
   별도 `section_content`로 검사하고 본문 비교와 분리했다.
2. `14a96c7e1b`: 76 PASS / 1 FAIL. HWPX의 기존 #1407/#1584 템플릿은 첫 문단에 단 정의가
   없으면 단일 단 정의를 추가한다. 아무 ColumnDef나 무시하지 않고 해당 경우만 정확히 하나의
   `column_count=1, same_width=true, 나머지 기본값`인지 검사하도록 정정했다.
3. `cc110f21f1`: Serialize 미구현 `DocInfo`를 JSON으로 비교하여 컴파일 실패했다.
   제품 타입을 변경하지 않고 테스트의 Debug 스냅샷 비교로 정정했다.
4. 최종 `698279af79`에서 전부 함께 재실행하여 78건 통과했다.

제품 저장기·파서·조판 규칙과 기존 baseline은 변경하지 않았다. 로그는
`output/3587/b3/focused-<SHA>.log`, `clippy-698279af7.log`,
`clippy-test-698279af7.log`(43.57초), `manifest-698279af7.log`다.
전체 회귀와 PR 직전 3종 Clippy 묶음은 이번 집중 검사와 별개다.

## 1/10/100회 비용 실측

프로브 SHA `14a96c7e1b`, `release-test` 프로파일. 이후 커밋은 테스트 비교만 변경했으며
제품 코드와 프로브 함수는 동일하다. `cost-14a96c7e1.log`에는 **서로 다른 9개 프로세스의
실제 테스트 1건 및 BLOCK_COST 1행씩**이 있다. 일반 CI에서는 프로브를 ignored로 둔다.

| 입력 | 사본 수 | 사전검사 µs | 반복 API µs | RSS 전→후 KiB | 프로세스 HWM 후 KiB | 반복 후 쪽 수 |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| 표 pi3 | 1 | 33 | 904 | 12,144→12,360 | 12,360 | 3 |
| 표 pi3 | 10 | 35 | 1,191 | 12,400→12,804 | 12,804 | 4 |
| 표 pi3 | 100 | 27 | 6,280 | 12,132→14,468 | 14,468 | 17 |
| 연구노트 pi12 | 1 | 37 | 441 | 11,596→11,920 | 11,920 | 3 |
| 연구노트 pi12 | 10 | 39 | 2,004 | 11,592→12,664 | 12,664 | 12 |
| 연구노트 pi12 | 100 | 37 | 18,290 | 11,800→19,864 | 19,864 | 102 |
| 글상자 pi0 — 거부 | 1 | 3 | 6 | 12,412→12,732 | 12,732 | 2 |
| 글상자 pi0 — 거부 | 10 | 3 | 6 | 12,276→12,596 | 12,596 | 2 |
| 글상자 pi0 — 거부 | 100 | 4 | 5 | 12,444→12,764 | 12,764 | 2 |

- 표: `samples/hwp_table_test.hwp`; 연구노트: `samples/rnote/labnote-001.hwp`.
- 글상자: `samples/table-in-tbox.hwp`. 세 요청 모두 `section/multicolumn paragraph boundary`
  unsupported이며 사본을 만들지 않았다. 성공 복제 성능으로 합산하지 않는다.
- 파일 로딩과 별도 preflight 뒤 repeat를 측정했다. repeat에는 내부 preflight와 정상
  페이지네이션이 포함된다. cold-start·평균/p95·최고 할당량 측정이 아니다.
- RSS 후 값은 결과 JSON 구성 과정에서 읽으므로 결과 포맷팅/allocator 영향도 포함할 수 있다.
  이 수치만으로 순수 복제 메모리나 메모리 누수 부재를 주장하지 않는다.
- 표 100회 논리 구조/대응표 예산: 1,068,400 / 444,800 bytes.
  연구노트 100회: 3,788,300 / 1,632,000 bytes. 각각 32 MiB / 8 MiB 정책 안이다.
  논리 예산은 RSS와 별개이며 이번 근거로 한도를 변경하지 않았다.

### 재실행 방법

review worktree에서 suite `--prepare` 후 `buildCaseIndex(deriveManifest())`로
`issue_3587_paragraph_block_save`의 target을 조회한다. 최종 SHA에서는
`regression_suite_019`지만 다른 SHA에 번호를 고정하지 않는다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --test regression_suite_019 -E 'test(/issue_3587_paragraph_block_save/)' \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
RHWP_3587_FIXTURE=labnote RHWP_3587_COUNT=100 \
  cargo test --locked --profile release-test --test regression_suite_019 \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  issue_3587_paragraph_block_save::block_cost_and_artifacts -- --exact --ignored --nocapture
```

fixture=`table/labnote/textbox`, count=`1/10/100` 각 조합을 별도 호출한다. 이번 실측에서는
`nextest list --list-type binaries-only --message-format json`의 정확한 binary를 직접
9회 실행했다(`binaries-14a96c7e1.json`). 0건 실행은 통과가 아니다.
`RHWP_3587_OUTPUT`은 count=1에서만 지정하고 존재하지 않는 명시적 경로를 사용한다.

## 메인테이너 확인용 산출물

모두 `output/3587/b3/` 아래에 있다.

| 폴더 | 내용 |
| --- | --- |
| `labnote-14a96c7e1/` | 원본 연구노트 2쪽, pi12 블록 1회 복제 결과 3쪽 |
| `table-14a96c7e1/` | 표 원본과 pi3 블록 1회 복제 결과, 각각 3쪽 |

`original.hwp`는 입력 원본, `repeated.hwp`·`repeated.hwpx`는 실제 저장 파일이다.
`original-NNN.svg`·`repeated-NNN.svg`, `mapping.json`, `measurement.json`도 있다.
SVG는 **저장 전 메모리 문서**의 렌더링이므로 저장 파일의 한컴 재열기 판정을 대신하지 않는다.
연구노트에는 내용을 아직 채우지 않았으며 빈 서식 복제 단계다.

에이전트가 연구노트 원본 2쪽/복제 3쪽 PNG를 열어 확인했다. 두 파일 SHA-256은
`9f8c9ff7927193ed8b3824c7bb7935ab35083228075773d62d8b2bb4be9487b6`로 동일하다.
한컴에서 `repeated.hwp`·`repeated.hwpx`가 정상적으로 열리고, 원본 앞뒤 서식 및 새 3쪽의
실험제목/목적·기록란·하단 서명란이 유지되는지 메인테이너 판정을 요청한다.

표 샘플 산출 과정에는 `LAYOUT_OVERFLOW` 4.5px/42.9px 경고가 남았다. 출력 로그만으로
원본 경고인지 복제 회귀인지 확정하지 않았으며 전체 시각 통과로 보고하지 않는다.

## 남은 순서

1. 생성 연구노트 HWP/HWPX의 메인테이너 실물 판정. 표 샘플 경고도 원본/결과를 구분한다.
2. B 종료 전체 회귀·workspace/WASM 포함 lint 통합 게이트.
3. B 결과 승인 후 C 내용 채우기/공개 API 계획·구현.
4. 이후 선택적 Gym에서 연구노트 내용 채우기·복제 시나리오 실행.

원격 push·PR·댓글, Gym 실행, WASM 재배포는 이번 절편에서 수행하지 않았다.
