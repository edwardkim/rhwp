# Task m100 #949 Stage 9 작업 보고서: TABLE probe HWP 생성

## 1. 목적

Stage 8에서 만든 TABLE probe manifest를 실제 판정 가능한 HWP 파일로 생성했다.

이번 단계의 파일은 HWPX 저장기 구현 결과가 아니다. generated HWP의 CFB 스트림을 기준으로
복사하고, BodyText TABLE 관련 record payload 일부만 oracle 값으로 graft한 판정용 파일이다.

## 2. 구현 내용

수정 파일:

```text
src/diagnostics/mod.rs
src/diagnostics/hwp5_table_probe.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-table-probe <oracle.hwp> <generated.hwp> \
  --out-dir <folder>
```

생성 방식:

```text
1. generated HWP의 모든 CFB stream을 읽는다.
2. BodyText/Section* stream을 압축 해제한다.
3. oracle/generated record를 LCS structural signature로 맞춘다.
4. TABLE/CTRL_HEADER(Table) record의 선택 축만 oracle payload로 graft한다.
5. BodyText를 다시 deflate 압축한다.
6. mini_cfb로 CFB를 재조립한다.
```

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/
```

생성 파일:

```text
01_ctrl_outer_margin_only.hwp
02_table_attr_only.hwp
03_table_tail_only.hwp
04_ctrl_common_attr_only.hwp
05_outer_margin_table_attr.hwp
06_outer_margin_table_tail.hwp
07_table_attr_tail.hwp
08_all_table_axes.hwp
stage9_generation.md
```

생성 명령:

```bash
./target/debug/rhwp hwp5-table-probe \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --out-dir output/poc/hwpx2hwp/task949/stage9/hwpx-h-01
```

## 4. 생성 결과

모든 파일은 rhwp reload 기준 9페이지로 열린다.

```text
01_ctrl_outer_margin_only: outer margin 26
02_table_attr_only:        table attr 19
03_table_tail_only:        table tail 26
04_ctrl_common_attr_only:  ctrl attr 9
05_outer_margin_table_attr: outer margin 26 + table attr 19
06_outer_margin_table_tail: outer margin 26 + table tail 26
07_table_attr_tail:         table attr 19 + table tail 26
08_all_table_axes:          outer margin 26 + ctrl attr 9 + table attr 19 + table tail 26
```

## 5. Stage 8 정정

Stage 7 `table-fields`에서는 `tail_after_0x16` 표시 길이 제한 때문에 14개 diff로 보였다.
Stage 9에서 full tail payload를 비교하니 TABLE 26개 모두 tail/length 차이가 있었다.

따라서 Stage 8 `table_probe_plan.md`도 full tail 기준으로 재생성했다.

## 6. 작업지시자 판정 대상

```text
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/01_ctrl_outer_margin_only.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/02_table_attr_only.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/03_table_tail_only.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/04_ctrl_common_attr_only.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/05_outer_margin_table_attr.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/06_outer_margin_table_tail.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/07_table_attr_tail.hwp
output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/08_all_table_axes.hwp
```

판정 항목:

```text
- 한컴 에디터에서 열리는지
- 이미지 출력이 유지되는지
- 표/셀 배치가 개선되는지
- rhwp-studio 재로드에서 표 배치가 개선되는지
```

## 7. 검증

```text
cargo build: 통과
hwp5-table-probe: 8개 HWP 생성 통과
rhwp reload: 8개 모두 pages=9
```

