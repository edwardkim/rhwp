# #7032 결과보고서 — 셀의 선행 빈 문단 줄 진행 보존

- Issue: #7032 — https://github.com/edwardkim/rhwp/issues/7032
- 작성: 2026-09-11 KST
- 브랜치: `task_m100_7032`
- 상태: **구현·로컬 회귀 완료, PR 제출 승인 대기. 병합·이슈 종료 전.**
- 근거: [수행계획](../plans/task_m100_7032.md), [구현계획](../plans/task_m100_7032_impl.md),
  [원인 조사](../working/task_m100_7032_stage1.md), [구현 결과](../working/task_m100_7032_stage2.md),
  [전체 검증](../working/task_m100_7032_stage3.md), [제출 준비](../working/task_m100_7032_stage4.md).

## 1. 해결 결과

HWP 표의 첫 제목 셀에서 편집자가 넣은 빈 문단을 실제 배치에도 반영했다.
기존에는 높이 측정에는 포함했지만 분할 표의 셀 배치에서 건너뛰어 `직렬`이 위로 당겨졌다.
수정 후 빈 문단의 줄 진행을 소비하며, 메인테이너의 SVG 시각 판정을 통과했다.

대상 HWP는 빈 문단 1개와 `직렬` 문단 1개를 저장한다. 파서의 누락이 아니며,
이미 정상인 대응 HWPX에는 저장된 줄 정보가 있어 같은 누락 경로를 타지 않았다.
구현 전후 HWPX 6쪽 SVG는 동일하고, HWP도 기존 6쪽·셀 높이·반복 제목행·대각선을 유지했다.
R2 확장 검증 이후에도 R1의 HWP/HWPX 12개 SVG가 바이트 단위로 동일했다.

## 2. 원인과 수정 경계

합성 줄(`ComposedLine`)이 0개라는 사실을 실제 빈 문단이 없거나 현재 페이지 소유가 아니라는
뜻으로 잘못 사용한 것이 원인이다. HWP 원시 레코드와 파싱 직후 IR에는 두 문단이 모두 남아 있었다.
조사 결과는 [Stage 1](../working/task_m100_7032_stage1.md)의 계보를 따르며, #7028의 대각선 수정이나
이번에 제공된 HWPX 파싱의 회귀로 분류하지 않는다.

- 온전한 셀과 잘린 셀에서 실제 빈 문단의 소유권을 기존 셀 콘텐츠 단위(`CellUnit`)로 판정한다.
- 소유한 빈 문단은 기존 문단 배치 fallback을 호출해 줄·캐럿 노드와 줄 진행을 함께 만든다.
- 부분 콘텐츠 높이, 마지막 가시 문단, 분할 여부가 동일한 소유권을 사용한다.
- 마지막 빈 문단의 높이는 기존 셀 측정 계약에 맞춰 뒤쪽 줄간격을 중복 소비하지 않는다.
- 텍스트가 없더라도 중첩 표·그림이 있는 문단은 단순 빈 문단으로 취급하지 않는다.

변경은 `paragraph_layout.rs`, `table_layout.rs`, `table_partial.rs`와 integration test 원본이다.
파서·저장기·IR 모델·전역 줄간격 해석은 변경하지 않았다. 파일명·확장자·문자열·고정 좌표로
예외 처리하지 않았으며, 일반 페이지네이션 재설계는 하지 않았다.

## 3. 신규 HWPX 검사 기준 등록

추가 샘플은 `samples/hwpx/21761835_jeonjik_exemption_table.hwpx`다.
원래 정상인 HWPX의 렌더링을 고친 것이 아니라, **기존 저장기가 재저장할 때 문단 ID를 새로
부여하는 특성**을 신규 샘플의 IR 왕복 검사 기준에 등록했다.

구현 전 `550d04840`과 구현 후의 전수 IR dump 260행은 동일했다. 미등록 차이는 셀 문단의
ID 바이트 1,432건, 본문 문단의 ID 바이트 24건이며 글자 소실 수가 아니다.
메인테이너 승인에 따라 이 샘플의 두 경로만 baseline에 추가했다(`4b74cf032`).
다른 샘플 기준값이나 검사기는 바꾸지 않았다. 등록 후 실측 dump도 등록 전과 동일하다.

## 4. 검증 결과와 한계

| 검증 | 실제 결과 | 기준 |
| --- | --- | --- |
| #7032 focused 회귀 | 7 PASS | R1/R2 구현 |
| 관련 focused 묶음 | 25 PASS, 1 기존 ignored | Stage 2 |
| fmt·native/WASM/workspace Clippy·workspace build | PASS | `63fe746c7` |
| Native Skia lib | 4,112 PASS, 13 ignored | `63fe746c7` |
| Native Skia placeholder / 직접 PDF | 2 / 4 PASS | `63fe746c7` |
| 최종 release-test 전체 nextest | **9,490 PASS, 0 FAIL, 46 skipped** | `4b74cf032`, 실행 422.106초 |
| 신규 HWPX 보안·IR 및 해당 코퍼스 래칫 | PASS, 전체 nextest 포함 | `4b74cf032` |
| suite manifest | 48/48 PASS | 등록 후 |
| 표준 Docker WASM 빌드 | PASS | `63fe746c7` |
| 실제 WASM API 출력 | HWP 6쪽 / HWPX 6쪽, 제목 텍스트 존재 | 빌드 산출물 smoke |
| 메인테이너 시각 판정 | R1 SVG PASS, R2 12개 SVG 동일 | 새 Docker 산출물의 직접 판정과 구분 |

최종 nextest 전의 1 FAIL은 신규 샘플 등록 전 결과다. 이를 감추거나 처음부터 성공한 것으로
기록하지 않으며 상세 대조는 Stage 3에 남겼다. `63fe746c7` 이후 제품·Rust test·Cargo 입력은
동일하고 baseline 두 행과 문서만 달라져 기존 lint·Skia·WASM 결과를 유지한다.

**미검증을 통과로 간주하지 않는다.** clipping controlset 외부 원본 92건이 모두 없어 해당
gate는 미검증이다. 기존 skipped/ignored도 PASS 수에 포함하지 않았다. WASM API smoke는
배치 좌표 시각 판정을 대신하지 않으며, 새 Docker 산출물의 메인테이너 직접 판정은 아직 없다.
동일 환경의 제품 성능 전후 비교는 미측정이다. 테스트 소요 시간을 성능 개선 수치로 쓰지 않는다.

## 5. 증적과 후속 절차

한컴 PDF는 기존 `pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf`를 재사용했다.
대표 표준 비교는 `output/7032/r1-after-hwp/cmp-p000.png`, 후속·마지막 쪽은 `cmp-p001.png`와
`cmp-p005.png`다. 로컬 로그는 `output/7032/stage3/`에 보존했다. 이 임시 경로를 공개 PR의
영구 증적 링크로 사용하지 않는다. 실제 PR 번호 발급 후 대표 이미지와 review 기록을 준비한다.

최신 `upstream/devel`은 `b5549292d8f6854fbc86ce825580dbe9496a567a`이며 기존 base 이후
10개 커밋이 추가됐다. 현 HEAD `df32b9dd5`와 merge-tree는 충돌 없이 성공했다.
같은 표 배치 파일에 캡션 소유자 전달 변경이 들어왔으므로, 텍스트 병합 성공을 통합 코드의
빌드·회귀 성공으로 해석하지 않는다. source branch에는 merge/rebase하지 않았다.

다음은 승인 후 push → `devel` 대상 Open PR → 번호 기반 증적·self-review 기록 → 최신 CI 확인이다.
충돌·필수 최신화가 없으면 단순 base 전진만으로 검증된 브랜치를 반복 병합하지 않는다.
병합 및 이슈 종료는 남아 있으며, 현재 보고서는 배포·통합 완료 선언이 아니다.
