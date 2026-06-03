# Task m100 #949 Stage 9 계획: TABLE probe HWP 생성

## 1. 목적

Stage 8 `table_probe_plan.md`에 정리한 TABLE 축을 실제 판정 가능한 HWP 파일로 생성한다.

이번 단계의 산출물은 구현 후보가 아니라 probe 파일이다. generated HWP를 기준으로 유지하되,
BodyText의 TABLE 관련 record payload 일부만 oracle 값으로 graft한다.

## 2. 구현 범위

새 CLI:

```bash
rhwp hwp5-table-probe <oracle.hwp> <generated.hwp> \
  --out-dir <folder>
```

생성 variant:

```text
01_ctrl_outer_margin_only
02_table_attr_only
03_table_tail_only
04_ctrl_common_attr_only
05_outer_margin_table_attr
06_outer_margin_table_tail
07_table_attr_tail
08_all_table_axes
```

## 3. 원칙

```text
1. generated HWP의 CFB 스트림을 기준으로 복사한다.
2. BodyText/Section* record를 압축 해제한다.
3. LCS로 oracle/generated record를 맞춘다.
4. TABLE/CTRL_HEADER(Table) record payload의 지정 축만 oracle 값으로 복사한다.
5. BodyText를 다시 deflate 압축하고 CFB를 재조립한다.
```

## 4. 검증

```text
cargo build
8개 HWP probe 생성
rhwp reload page_count 확인
```

