# 구현계획서 — Task M100-1304: 수식 무브레이스 첨자 공백/연결 구분

## 0. 수행계획서 대비 방향 보정 (중요)

수행계획서 기본안은 **후보 C(토크나이저가 일반 공백을 비가시 경계 토큰으로 보존 + 파서가 경계까지 run 소비)** 였다.
구현 설계 + 권위 PDF(한글 2022) 검증을 통해 **최소 변경 설계**로 보정했다.

### 0.1 권위 PDF 검증 결과

`pdf/3-10월_교육_통합_2022.pdf` 11쪽 문18 확인:
- `sum_k=1 ^6` → ∑ **아래 `k=1`(전체), 위 `6`** 으로 렌더. → 무브레이스 하한이 `k=1` 전체임이 확정.
- `k^2-4` → `2` 는 위첨자(=`2-4` 아님). → 산술연산자(`-`)는 operand 를 확장하지 않음.
- `x^2 = 4` 류 등식은 **공백으로 구분**. → 공백 정보가 필요. 관계연산자만으로 묶으면 `x^2 = 4` 를 `x^{2=4}` 로 깨뜨림.

### 0.2 최종 설계 (블래스트 반경 최소)

후보 C 의 "경계 토큰 + 모든 루프 boundary-skip" 은 영향이 과도하다 (`parse_single_or_group` 25개 호출처, 모든 공백 루프). 대신:

> **(1) 토크나이저**: `Token` 에 `space_before: bool` **플래그 한 개만** 추가 (경계 토큰 X). 토큰 생성처가 `tokenizer.rs` 한 파일에만 있어 완전 격리.
>
> **(2) 파서**: 무브레이스 **아래첨자(하한) operand 전용** 파서 `parse_script_operand` 도입. `원자 (관계연산자 원자)*` 패턴으로 묶되, **관계연산자 앞에 공백(`space_before`)이 있으면 묶지 않는다**.
>
> **(3) 적용 범위 제한**: 관계연산자 병합은 **아래첨자(`_`)/극한 하한에만** 적용. **위첨자(`^`)는 기존 단일 atom 유지** → `x^2=4` 류 위첨자 등식을 원천적으로 미접촉.

근거:
- 모든 버그 케이스(`sum_k=1`, `lim_x->0`, `1<=k`)가 **하한(아래첨자)** 이다. 위첨자 병합은 불필요하며 위험만 추가.
- `space_before` 플래그가 `x^2 = 4`(공백 등식)를 보호하고, 아래첨자 한정이 위첨자 등식을 추가 보호.
- 토크나이저는 토큰 1개당 `space_before` 만 스탬프 (glue-split 꼬리 토큰은 contiguous → false, 정상).
- 명령 body(`sqrt`/`bar`/`over` 등) 단일 토큰 동작 무변경. `a_n b_m`(인접 식별자) 무변경.

## 1. 근본 원인 (재확인)

- HWP 수식에서 무브레이스 첨자 operand 는 공백/다음 연산자까지 확장된다. `sum_k=1 ^6` = ∑(하한 `k=1`, 상한 `6`).
- 현재 `parse_single_or_group` 는 무브레이스 operand 를 **단일 토큰**만 취함 → `sub=k`, `=1` 은 본문, `^6` 은 `1` 의 위첨자로 오파싱.

## 2. 설계

### 2.0 토크나이저 — `space_before` 플래그

```text
Token { ty, value, pos, space_before: bool }   // 필드 1개 추가
Tokenizer { ..., last_had_space: bool }         // 보조 상태
```

- `next_token` 진입 시 `skip_spaces` 전후 pos 델타로 `last_had_space` 설정.
- `tokenize()` 루프에서 반환 토큰에 `token.space_before = last_had_space` 스탬프.
- glue-split 꼬리 토큰(`sinx`→`sin`,`x`)은 contiguous 라 `space_before=false` (정상).

### 2.1 신규 함수 `parse_script_operand()` (아래첨자/하한 전용)

```text
operand := atom ( REL_SYMBOL atom )*
atom    := parse_single_or_group()  (브레이스 그룹 또는 단일 bare 토큰; try_parse_scripts 미호출)
REL_SYMBOL := Symbol("=" | "<" | ">" | "<=" | ">=" | "!=" | "==" | "->")  AND space_before == false
```

- `{` 로 시작하면 첫 atom 에서 `parse_group()` 위임 (브레이스 표기 100% 유지).
- run 지속 조건: 다음 토큰이 **공백 없는 관계연산자**일 때만. 그 외(`^`/`_`/괄호/산술연산자/명령/식별자/공백 후 토큰/EOF)에서 정지.
- `->` 는 `MathSymbol("→")` 로 변환해 push (parse_element 와 동일).
- run 1개면 단일 노드, 2개+면 `Row(...).simplify()`.

`space_before` 가드: `x^2 = 4` 처럼 관계연산자 앞 공백이 있으면 묶지 않음. `sum_k=1`(공백 없음)만 묶음.
관계연산자 한정: `k^2-4`(산술 `-`)는 확장 안 함. `a_n+1` 도 기존대로 `n` 만.

### 2.2 적용 지점 (아래첨자/하한만)

| 위치 | 현재 | 변경 |
|------|------|------|
| `try_parse_scripts` **sub** (665) | `parse_single_or_group` | `parse_script_operand` |
| `parse_big_op` **sub** (1115) | `parse_single_or_group` | `parse_script_operand` |
| `parse_limit` **sub** (1133) | `parse_single_or_group` | `parse_script_operand` |

**위첨자(`^`) 및 명령 body 는 변경하지 않는다**: try_parse_scripts sup(669), big_op sup(1118), `sqrt`/`bar`/`over`/`overset`/`color`/`matrix`/LSUB·LSUP 등 모두 기존 `parse_single_or_group` 유지.
이로써 `x^2=4`(위첨자 등식)를 원천적으로 미접촉.

### 2.3 비범위 (별도 추적)

- `(k+1)^2` 의 위첨자 base 가 비는 **orphan superscript on Paren** 증상은 본 이슈와 별개의 사전 결함이다 (브레이스 표기 `(k+1) ^{2}` 에서도 동일 발생). 본 task 에서 수정하지 않고 **후속 이슈로 분리**한다. 문18 시각상 `2` 위치가 남을 수 있음을 보고서에 명시한다.
- 토크나이저 공백 처리(#505) 변경.

## 3. 구현 단계

### Stage 1 — 회귀 고정 테스트 작성 (red)

`parser.rs` 테스트 모듈에 baseline/대상 테스트 추가:

- 대상(현재 실패): `sum_k=1 ^6` → `BigOp{∑, sub=Row[k,=,1], sup=6}` 기대
- 추가 대상: `lim_x->0`, `prod_i=1 ^n`, `sum_1<=k<=n ^{}`(관계 연쇄)
- 회귀 보호(현재 정상 유지): `sum _{k=1} ^{6}`(브레이스), `x^2`, `a_n`, `a_n^2`, `a_n b_m`(인접 식별자 분리), `a_n+1`(arithmetic 미병합)

이 단계는 테스트만 추가하며, 대상 테스트는 일단 실패(또는 `#[ignore]` 없이 기대값 명시)로 둔다.

### Stage 2 — `parse_script_operand` 구현

- 기존 `parse_single_or_group` 의 단일 토큰 분기를 `parse_atom()`(가칭)으로 추출(또는 내부 재사용).
- `parse_script_operand` 구현: 2.1 의 `atom (REL_SYMBOL atom)*` 패턴.
- 첨자 3개 지점(2.2)을 `parse_script_operand` 로 교체.
- Stage 1 의 대상 테스트 green, 회귀 테스트 green 확인.

### Stage 3 — 회귀·시각 검증

- `cargo test --lib equation`, `cargo test --lib`, `cargo test --tests` 전체 통과.
- 시각: `rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 10` → 문18 시그마 4줄 상·하한 정상 배치 확인 (PNG 캡처).
- orphan superscript(`(k+1)^2`) 잔존 여부 기록 → 후속 이슈 등록 판단.

### Stage 4 — 최종 보고 (필요 시 Stage 3 에 통합)

- 단계별 보고서 + 최종 결과 보고서 작성, 오늘할일 갱신.

> 본 task 는 변경 범위가 좁아 **3단계**(Stage 1~3)로 운용하고, Stage 4 보고는 Stage 3 에 통합 가능. 단계별 보고서는 각 stage 커밋과 함께 기록한다.

## 4. 변경 파일 요약

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/parser.rs` | `parse_script_operand` 신규, 첨자 3지점 교체, 테스트 추가 |
| `mydocs/working/task_m100_1304_stage*.md` | 단계별 결과 |
| `mydocs/report/task_m100_1304_report.md` | 최종 보고서 |

## 5. 검증 명령

```text
cargo test --lib equation
cargo test --lib
cargo test --tests
rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 10
```

## 6. 완료 기준

```text
1. sum_k=1 ^6 → BigOp{∑, sub=Row[k,=,1], sup=6} 파싱.
2. 브레이스/단일첨자/인접식별자/arithmetic 첨자 회귀 없음 (테스트로 고정).
3. 문18 시그마 4줄 상·하한 정상 렌더.
4. cargo test --lib / --tests 전체 통과.
5. orphan superscript(괄호 위첨자)는 후속 이슈로 분리 기록.
```

## 7. 승인 요청

위 구현계획(보정된 기본안: 외과적 관계연산자 run, 토크나이저 무변경)으로 Stage 1 부터 진행한다.
방향 보정(후보 C → 외과적 변형)에 대한 승인을 함께 요청한다.
