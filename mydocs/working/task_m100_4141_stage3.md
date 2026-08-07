# Stage 3 — task_m100_4141 한컴 판정 번들과 후속 이슈 초안

- **이슈**: [#4141](https://github.com/edwardkim/rhwp/issues/4141)
- **선행 단계**: [stage1](task_m100_4141_stage1.md) · [stage2](task_m100_4141_stage2.md)
- **최종 보고서**: [`mydocs/report/task_m100_4141_report.md`](../report/task_m100_4141_report.md)
- **작업 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: 0

## 1. 한컴 판정 번들 (`output/issue_4141/`, 비커밋)

`.gitignore:15` 의 `/output/` 로 제외되므로 산출물은 커밋되지 않는다. 수치만 보고서로 옮긴다.

```text
output/issue_4141/
  PANJEONG.md              판정 안내 (#4097 판형 답습)
  measure_spans.py         PyMuPDF span 크기 계측 + A1~A5 자동 판정
  SO-SUEOP-before.hwp      수정 전 rhwp convert
  SO-SUEOP-after.hwp       수정 후 rhwp convert
  SO-SUEOP-before.hwpx     수정 전 rhwp export-hwpx
  SO-SUEOP-after.hwpx      수정 후 rhwp export-hwpx
```

`before` 는 분기 기준(`upstream/devel` `0fdac31ba`, 수정 전) 빌드로, `after` 는 수정 후 빌드로
**같은 명령**을 실행해 만들었다.

### 인계 전 바이트 사전검증 — 통과

| 파일 | CHAR_SHAPE / relSz 태그 | relSz | ratios(장평) | base_size 최빈 |
| --- | ---: | --- | --- | --- |
| `before.hwp` | 2,512 | **0 × 2,512** | 100 × 977 | 1000 × 1706 |
| `after.hwp` | 2,512 | **100 × 2,512** | 100 × 977 | 1000 × 1706 |
| `before.hwpx` | 2,512 | **0 × 2,512** | — | — |
| `after.hwpx` | 2,512 | **100 × 2,512** | — | — |

**격리된 변수는 상대크기 하나다** — CHAR_SHAPE 개수·장평·기준 크기가 before/after 동일하다.

- `.hwpx` 는 12개 ZIP 엔트리 중 **`Contents/header.xml` 하나만** 다르다.
- `.hwp` 는 총 크기 99,840 으로 동일하고 6,227바이트가 다르다 — DocInfo 가 deflate 압축이라
  2,512 × 7바이트 변경이 스트림 전체로 번지기 때문이다.

```text
29fa7bf2b3f3ded72d11785545f251031289b3be0c0e5637262dfdae7a23a2af  SO-SUEOP-before.hwp
6a89786e273e6cc6da6f43bae77408c2a0698886411e474e79d97cbd51b7229d  SO-SUEOP-before.hwpx
cbe7d76c7b215e25e3db2c326d50469f2e59076af48ce335428cc9ba54827558  SO-SUEOP-after.hwp
7d763c85ad04216282baa3f355bd319ee2d6da33acc30e9b1ea0408b1a047c8c  SO-SUEOP-after.hwpx
```

## 2. 한컴 판정 결과 (2026-08-07 수행)

작업지시자가 한컴으로 열어 PDF 로 출력했다 —
`output/issue_4141/SO-SUEOP-{before,after}.pdf`. PyMuPDF 계측 결과:

| | 쪽수 | span | min | max | median | **1pt 미만** |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 원본 `pdf/SO-SUEOP-2024.pdf` | 46 | 14,417 | 7.17 | 50.64 | **9.71** | 0 |
| `before` (수정 전) | 46 | 10,604 | 0.12 | 0.12 | 0.12 | **10,604** |
| `after` (수정 후) | **47** | 7,730 | 6.96 | 50.51 | **9.71** | **0** |

### 합격 기준 판정

| # | 기준 | 결과 |
| --- | --- | --- |
| A1 | `after` 의 1pt 미만 span = 0개 | **PASS** — 0개 |
| A2 | 크기 분포 min/max/median ±5% | **PASS** — min −2.9%, max −0.3%, median **0%**(9.71 일치) |
| A2′ | 최빈 8종 일치 | **FAIL** — 원본은 9.71 이 8,426개로 압도적, `after` 는 8.54/9.71/9.72 로 분산 |
| A3 | `after` 쪽수 = 46 | **FAIL** — 47쪽 |
| A4 | 1쪽 육안 정상 | **FAIL** — 아래 §3 |
| A5 | **음성 대조군** — `before` 재현 | **PASS** — 10,604 span 전부 0.12pt. 이슈 본문 수치와 **정확히 일치** |

**A5 가 정확히 재현됐으므로 판정 절차 자체는 유효하다.** 결과를 채택한다.

### 판정 — #4141 의 계약은 해소됐다

이 이슈가 정의한 결함은 "본문 글자가 전부 ~0.1pt 로 그려진다"이고, 그것은 **해소됐다**:
1pt 미만 span 10,604개 → **0개**, median 이 원본과 소수점까지 일치(9.71).
본문도 온전하다 — 추출 문자수 원본 50,172자, `after` 54,219자.

**다만 문서는 아직 사용 가능하지 않다.** A3·A4 실패의 원인은 #4141 과 별개인 결함 둘이다(§3).

## 3. 판정에서 드러난 별개 결함 둘

### ① 글자 음영이 검정 — [#4155](https://github.com/edwardkim/rhwp/issues/4155) (신규 등록)

`after` 는 **본문 전체가 검정 막대**로 덮인다. PDF 를 뜯어보면 텍스트는 제 위치·제 크기로
그려져 있고 그 위에 줄 크기 순검정 사각형이 칠해져 있다(3쪽 fill 65개. 원본 같은 쪽은
글리프 크기 fill 35개).

**#4141 과 인과가 없다** — `shade_color` 바이트가 before/after **동일**하다(둘 다
`0x00000000` × 2,512). #4141 이전에는 글자가 0.12pt 라 음영 사각형도 찌부러져 있었을 뿐이다:

```text
before 3쪽 검정 fill: 3개, 크기 0×0
after  3쪽 검정 fill: 65개, 줄 크기
```

근인 둘 — HWP3 `shade_ratio`(음영 비율) 미반영, HWP5 라이터의 "음영 없음" sentinel 미번역.
실측 근거는 이슈 본문에 전부 담았다(한컴산 HWP5 380건 코퍼스, HWP3 `shade_ratio` 임시 프로브,
같은 원본에 대한 한컴 저장본 4쌍 대조).

**작업지시자 판단(2026-08-07): #4141 merge 이후 별도 작업.** 이 브랜치에서는 다루지 않는다.
계약 테스트 설계는 이슈 §"수정 방향" 3번과 실측 표에 값까지 명시돼 있어 그대로 재작성 가능하다.

### ② 1쪽 글맵시 누락 — [#4097](https://github.com/edwardkim/rhwp/issues/4097) 축

원본 1쪽의 세로 글맵시 제목("수업용소설해설")이 `after` 에 없다. 그 수정은
[PR #4144](https://github.com/edwardkim/rhwp/pull/4144)(`task_m100_4097`)에 있고, 이 브랜치는
`upstream/devel` 에서 분기해 **미포함**임을 확인했다(`git merge-base --is-ancestor` 실패).

`pdf/task4097/README.md` 가 "#4141 해소 후 같은 쌍을 다시 만들면 이 축의 **양성 판정**이 비로소
가능해진다"고 적어 뒀다 — 이 수정이 그 전제를 풀었다.

### A3(쪽수 47 vs 46) 는 원인 미확정

`before` 는 전 글자가 0.12pt 라 조판이 무의미하므로 그 46쪽은 레이아웃 기준선이 될 수 없다.
`after` 47쪽이 원본 46쪽과 다른 것은 HWP3 → HWP5 **조판 충실도** 축인데, #4155·#4097 이
남아 있는 상태에서는 기여를 분리할 수 없다. **세 결함이 해소된 뒤 재측정해야 판단이 선다** —
지금 단정하지 않는다.

## 4. 원래 합격 기준 (사전 고정분, 참고)

| # | 기준 |
| --- | --- |
| A1 | `after` 의 1pt 미만 span = 0개 (before 는 10,604개 전부 0.12pt) |
| A2 | `after` 크기 분포가 원본과 정합 — min/max/median ±5%, 최빈 8종 일치 |
| A3 | `after` 쪽수 = 46 = `before` = 원본 |
| A4 | 1쪽 육안 정상 |
| A5 | **음성 대조군**: `before` 재현 — 10,604 span 전부 ≈0.12pt |

## 5. 후속 이슈 초안 — `ratios` 기본값

작업지시자 승인 후 등록한다. GitHub 권한상 label·assignee 는 지정하지 못한다.

---

**제목**

```text
[model] CharShape.ratios 기본값 0 이 OWPML 유효범위 [50,200] 밖 — 렌더러가 장평 0 으로 소비한다
```

**본문**

```markdown
> [#4141](https://github.com/edwardkim/rhwp/issues/4141) 수정 중 발견해 분리 등록합니다.
> 같은 부류(모델 파생 기본값이 스펙 유효범위 밖)이지만 **렌더 가시 변경이라 검증 lane 이 다릅니다.**

## 증상

`RELSIZE`/`RATIO` 자식이 없는 HML 을 배포 CLI 로 왕복시키면 장평이 0 으로 나갑니다.

    rhwp export-hml tests/fixtures/hml/exambank_math_equations_min.hml -o /tmp/rt.hml

    <RATIO Hangul="0" Latin="0" Hanja="0" Japanese="0" Other="0" Symbol="0" User="0"/>

입력 fixture 의 `<CHARSHAPE>` 는 `FONTID` 만 갖고 `RATIO` 자식이 없습니다.

## 원인

`CharShape.ratios` 의 기본값이 `[0; 7]` 입니다. OWPML `ratio` 는 default=100,
유효범위 [50, 200] 이므로(`mydocs/manual/OWPML SCHEMA/Header XML schema.xml`) 0 은 범위 밖입니다.

`relative_sizes` 와 달리 **렌더러가 이 값을 소비합니다** —
`src/renderer/style_resolver.rs:355` `ratios.push(cs.ratios[lang] as f64 / 100.0)`.
장평 0 = 글자 폭 0 입니다.

값을 채우지 않는 경로:

- `src/parser/hwpx/header.rs:588` — `charPr` 에 `<hh:ratio>` 자식이 없을 때
- `src/parser/hwpx/header.rs:848-858` — charPr id 갭 `resize_with(idx+1, CharShape::default)`
- `src/parser/hml/reader.rs:599-605` — `RATIO` 부재
- `src/document_core/html_table_import.rs:627`, `src/document_core/commands/html_import.rs:845`
- `src/parser/hwp3/mod.rs:3566` — 인덱스 0 placeholder

## HWP3 축은 안전합니다 (#4141 과 인과 없음)

- `convert_char_shape`(`src/parser/hwp3/mod.rs:540`)가 `ratios` 를 HWP3 레코드에서 채웁니다.
- `ratios=0` 인 유일한 레코드는 인덱스 0 placeholder 인데, **어떤 문단도 참조하지 않습니다** —
  HWP3 표본 15건 전수에서 `PARA_CHAR_SHAPE` 참조 0건 (SO-SUEOP 은 참조 1,918건 중 최소 id 가 16).

그래서 #4141 의 백지 증상과는 무관하고, 그 PR 에 묶지 않았습니다.

## 수정 방향 (제안)

1. `impl Default for CharShape`(`src/model/style.rs`, #4141 에서 도입)의 `ratios` 를
   `[100; 7]` 로. `base_size` 도 같은 축이라 함께 검토합니다.
2. #4141 이 남긴 `char_shape_default_matches_spec_only_for_relative_sizes` 단언을 갱신 —
   그 테스트가 지금 `ratios == [0;7]` 을 "의도된 미수정"으로 고정하고 있습니다.
3. HML/HWPX 왕복 계약에 `RATIO` 유효범위 단언 추가
   (`tests/issue_4141_hwp3_relative_size_contract.rs` 의 범위 검사 헬퍼 재사용 가능).

## 검증 부담 (분리한 이유)

렌더러가 소비하므로 `local_validation.md` 4.3 의 **renderer lane** 이 발동합니다 —
Native Skia 3종 + `wasm-pack build` + 시각 증적. `relative_sizes`(렌더 참조 0건)와 달리
골든 스냅샷이 움직일 수 있습니다.

## 관계

- [#4141](https://github.com/edwardkim/rhwp/issues/4141) — 같은 모델 기본값 결함의 `relative_sizes` 축.
  근거 계측: `mydocs/working/task_m100_4141_stage1.md` §3, §5, §7
```

---

## 6. 절차 상태

- 한컴 판정 완료(2026-08-07). **#4141 의 계약은 해소**됐고, 판정에서 드러난 별개 결함
  [#4155](https://github.com/edwardkim/rhwp/issues/4155) 를 실측 근거와 함께 등록했다.
- **#4155 는 #4141 merge 이후 별도 작업**한다(작업지시자 판단, 2026-08-07). 이 브랜치의 범위는
  `relative_sizes` 하나로 유지한다 — 한컴 판정 결과의 귀속을 흐리지 않기 위해서다.
- 이 단계에서 임시 프로브(`convert_char_shape` 의 `shade_ratio` 출력)를 썼고 **측정 후 삭제**했다.
  프로덕션 코드 변경 0 을 `git status` 로 확인했다.
- 오늘할일(`mydocs/orders/`)은 **contributor 가 작성하지 않는다** — 이 작업의 산출물에서 제외한다
  (작업지시자 지적, 2026-08-07). 2026-08-07 갱신된 `collaborator_self_merge.md` §8.2.1 도
  오늘할일·`pr_N_review.md` 를 **PR 채번 이후** 단계로 옮겼고, 그 경로는 collaborator 몫이다.
  PR 번호를 예측해 `pr_N_*` 파일명을 미리 만들지도 않는다.
- **remote push 와 PR 생성은 작업지시자 승인 대기**다
  (`docs_and_git_workflow.md:166-184`, `AGENTS.md:31`). 2026-08-07 갱신된 규칙상 구현·로컬
  검증이 끝난 merge 후보는 별도 Draft 지시가 없으면 **Open PR** 로 생성한다.
- 전체 `cargo test --profile release-test --tests` 와 `cargo clippy --all-targets -- -D warnings` 도
  별도 승인 대기다(`docs_and_git_workflow.md:181-184`).
- `ratios` 후속 이슈(§5) 등록은 승인 대기다.
