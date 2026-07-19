---
kind: snapshot
status: historical
canonical: mydocs/manual/codex/README.md
last_verified: 2026-07-17
---

# Task M100 #854 Memory

## Task Identity

- 이슈: Task #854
- 최근 작업 브랜치였던 곳: `local/task_m100_854`
- 현재 브랜치: `local/devel`
- 마지막 안정 커밋으로 기억되는 지점:

```text
a510c6c1 fix: complete task 854 rebuild stage 4
```

## Clean Reset Memory

사용자 지시로 오염된 작업 브랜치를 clean했다.

실행된 정리의 의미:

- Stage 6에서 시도했던 실패성 코드 변경은 되돌림
- untracked probe example/doc 일부 삭제
- generated output은 git ignored라 일부 남아 있을 수 있음

삭제 대상으로 기억되는 파일:

```text
examples/hwp_para_text_dump.rs
examples/hwpx_mel_probe_variants.rs
mydocs/working/task_m100_854_rebuild_stage5.md
mydocs/working/task_m100_854_rebuild_stage6.md
mydocs/working/task_m100_854_rebuild_stage6_subplan.md
samples/hwpx/hancom-hwp/tac-table-01.hwp
samples/hwpx/tac-table-01.hwpx
```

현재 `local/devel`에서 실제 존재 여부는 재확인해야 한다.

## Stage 1 To 4 Memory

Stage 1:

- `basic-table-01` 손상 조건 추적
- 현재 `export_hwp_with_adapter()`는 저장용 clone이 아니라 live IR을 mutate한다는 관찰이 있었다.
- 단, 당시 손상 원인 추적은 clone 여부보다 HWP5 record materialization 차이가 우선이었다.

Stage 2:

- 문단 배경 무늬 ordinal 문제가 있었다.
- 한컴 에디터에서 문단 > 배경 > 무늬모양이 2번째로 지정되는 문제.
- HWP와 HWPX 사이에서 무늬 종류 기수 차이를 의심.
- 스펙과 한컴 도움말 교차 검증 대상이 됨.

Stage 3:

- `basic-table-01`의 `PARA_HEADER` 정합.
- HWP5 `PARA_HEADER` 크기와 관련된 검증.

Stage 4:

- `expense_report.hwpx` 사용.
- 쪽 배경 관련 `PAGE_BORDER_FILL` 검증.
- 처음에는 쪽 배경까지 그린 후 손상 판정.
- 이후 파일 손상은 사라졌지만 표 배치 비정상.
- Stage 4 구현으로 표 배치까지 해결됨.
- 사용자가 한컴 에디터에서 정상 렌더링을 확인.

## Stage 5 Memory

대상:

```text
samples/hwpx/tac-table-01.hwpx
samples/hwpx/hancom-hwp/tac-table-01.hwp
```

사용자 검증:

- 한컴 에디터에서 정상적으로 열림
- 조판도 잘 됨
- 페이지 수는 사용자가 시각 검증으로 확인하기로 함

## Stage 6 Memory

대상:

```text
samples/hwpx/mel-001.hwpx
samples/mel-001.hwp
```

증상:

- 한컴 에디터에서 2페이지까지 렌더링 후 파일 손상 판정
- rhwp-studio에서는 20페이지가 정상으로 보였음
- 마지막 20페이지 이미지 배치가 잘못됨
- 중간에 2페이지 표 배경이 검정색으로 나오는 문제가 있었으나 이후 정상 처리됨
- 그래도 2페이지 파일 손상은 계속 발생
- 사용자는 "3 예산현황 다음 표" 근처에서 문제가 생기는 것 같다고 판단

사용자 불만의 핵심:

- POC에서는 이 예제가 잘 되던 것이 현재 구현에서는 오류를 일으킨다.
- Codex가 엉뚱한 곳을 고치고 있는 것 아니냐는 의심이 합리적이다.
- 현재 문제는 parser/IR/rendering이 아니라 clone 후 저장 과정의 누락 또는 오매핑이다.

## Failed Variants Memory

다음 변형들은 한컴에서 동일하게 파일 손상 판정:

```text
output/poc/hwpx2hwp/task854/mel_probe_variants/12_first_para_ctrl_text_order_tab_ext.hwp
output/poc/hwpx2hwp/task854/mel_probe_variants/13_newnum_page_control_code.hwp
output/poc/hwpx2hwp/task854/mel_probe_variants/14_cell_width_ref_refish.hwp
output/poc/hwpx2hwp/task854/mel_probe_variants/15_fixed_width_space_code.hwp
```

따라서 이 방향을 무작정 재시도하지 않는다.

## POC Artifacts To Compare

남아 있을 가능성이 있는 POC 산출물:

```text
output/poc/hwpx2hwp/task854/mel_probe_variants/poc_stage18_mel-001.hwp
output/poc/hwpx2hwp/task854/mel_probe_variants/poc_stage39_mel-001.hwp
output/poc/hwpx2hwp/task854/rebuild_stage6_mel_probe/mel-001.hwp
output/poc/hwpx2hwp/task854/mel_probe_variants/
```

이 파일들은 git ignored 산출물일 수 있으므로 존재 여부를 재확인한다.

## Correct Next Approach

다음 재시도는 구현이 아니다.

해야 할 순서:

1. `local/devel`에서 현재 기준 확인
2. 관련 트러블슈팅 검색
3. POC 성공 HWP와 현재 실패 HWP와 한컴 정답 HWP를 구조 비교
4. 차이 항목을 record 단위로 좁힘
5. 저장 materialization 경로의 누락 후보를 문서화
6. 계획서 작성
7. 사용자 승인 후 구현

비교 관점:

- DocInfo ID table
- BinData ID/index
- SectionDef/PageDef/PageBorderFill
- PARA_HEADER/PARA_TEXT/LIST_HEADER
- TABLE record trailer
- cell LIST_HEADER 확장부
- control char와 control record 순서
- `char_count` vs UTF-16 code unit
- `control_mask` vs controls
- HWPX ordinal과 HWP ordinal 차이
