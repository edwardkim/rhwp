---
kind: report
status: done
canonical: mydocs/report/archives/task_m100_5967_report.md
last_verified: 2026-08-24
---

# #5967 보고서 — 판정 자산의 재생성 계약을 바이트에서 스트림으로 옮긴다

- **이슈**: [#5967](https://github.com/edwardkim/rhwp/issues/5967)
- **계획서**: [`task_m100_5967.md`](../../plans/archives/task_m100_5967.md) · **Stage 1**:
  [`task_m100_5967_stage1.md`](../../working/archives/task_m100_5967_stage1.md)
- **브랜치**: `task5967` (`upstream/devel` `ad2867708` 기준)
- **판정 원장**: [`samples/issue5447/MANIFEST.json`](../../../samples/issue5447/MANIFEST.json)

## 0. 결론

**라이터 버그가 아니다. 원인은 `b9eb55107` 이고, 되돌리지 않는다.** 판정 자산은 동결하고,
잃어버린 "재생성 바이트 동일성" 을 **"재생성 스트림 동일성"** 으로 대체해 CI 상시 계약으로
승격했다. `src/` 는 한 줄도 바뀌지 않는다.

## 1. 원인 — 이슈가 열어 둔 "원인 창" 을 닫았다

`src/serializer/mini_cfb.rs` 를 #5647 병합(`e555f759a`, 2026-08-20) 이후에 건드린 커밋은
하나뿐이다.

**`b9eb55107` (2026-08-22) — "검토: CI 완료 외부 PR 7건 통합 수용 (#5912)"**. 이 통합 커밋이
외부 PR 두 건을 cherry-pick 하며 라이터를 66줄 늘렸다.

| cherry-pick | 코드 | 바이트 |
|---|---|---|
| `6a0c04159` "emit valid CFB directory trees" | `DirEntry.color` 신설, `color_deepest_nodes()`, `write_dir_entry` 가 상수 `1` 대신 `entry.color` 기록 | 디렉터리 엔트리 `+0x43` `01` → `00` |
| `97dbf4cef` "initialize unused CFB slots with sentinels" | FAT·MiniFAT 섹터 FREESECT 선채움, 미할당 디렉터리 슬롯 NOSTREAM | FAT·MiniFAT 꼬리 `00` → `ff` |

## 2. 왜 되돌리지 않는가

두 변경은 MS-CFB 방향으로 옳다. 종전 "전 엔트리 black" 은 leaf 깊이가 갈리는 트리에서
black-height 가 어긋나고, FAT 미사용 슬롯의 `0x00` 은 strict 파서에서 "다음 섹터가 0번" 으로
읽힌다(섹터 0 은 언제나 사슬 머리라 후속이 될 수 없다).

게다가 **같은 커밋이 반대 방향을 이미 고정한다** —
[`tests/cases/mini_cfb_strict_contract.rs`](../../../tests/cases/mini_cfb_strict_contract.rs) 의
`directory_tree_obeys_red_black_invariants`(스트림 1~128 전수 black-height·red-red 인접·root
black)와 `unused_container_slots_use_required_sentinels`. 되돌리면 이 둘이 즉시 빨개지고 외부
기여 PR 2건이 무효화된다.

유효성 문제도 관측되지 않았다 — 현재 라이터 포장 상태의 산출 32건을 한글 2022 가 전건 정상
개봉·렌더했다(#5652 판정, 개봉 실패 0).

**그러므로 `samples/issue5447/` 와의 바이트 동일성은 회복 불가다.** 자산을 다시 만들어 회복하는
길도 택하지 않았다 — 재생성하면 `pdf/issue5447/` 의 한컴 PDF 와 짝이 끊어져 판정 자체를 다시
받아야 하고, 그 순간 #5447 의 역사적 판정 증적이 사라진다.

## 3. 만든 것 — 라이터 판을 넘어 성립하는 두 계약

[`tests/cases/issue_5967_cfb_repack_reproducibility.rs`](../../../tests/cases/issue_5967_cfb_repack_reproducibility.rs)
(신규). 생성기(`b2_variants`)에 기대지 않고 커밋된 자산만으로 자기완결하며 `output/` 을 쓰지
않아 CI 에서 상시로 돈다. 자산 목록·역할은 원장에서 읽는다(파일명 하드코딩 금지 — #5447 §6-1
NFC/NFD 사고 선례).

### 3-1. `judged_streams_survive_a_current_writer_repack` — 38건

중첩 CFB 를 `all_ole_streams` 로 열거해 `build_cfb_with_root_clsid` 로 **지금 라이터가** 재조립한
뒤, 스트림 이름·순서·바이트와 루트 CLSID 가 전부 보존되는지 본다. 대조군 9건(한컴 저작 CFB)도
함께 봐서 원본 작성기가 누구든 성립함을 고정한다.

즉 **한컴이 판정한 내용은 재현된다.** 바뀐 것은 컨테이너 살림뿐이라는 이슈의 관측이 이제
데이터가 아니라 계약이다.

### 3-2. `writer_drift_is_confined_to_directory_color_and_fat_fill` — 29건

커밋본과 재조립본을 바이트로 대조하고 차이를 두 버킷으로 분류한다.

- **색 플래그** — 디렉터리 섹터 안 엔트리 `+67`, `01` → `00`
- **할당표 미사용 슬롯** — FAT·MiniFAT 섹터 안 4바이트 슬롯, `0x00000000` → FREESECT

두 버킷 밖 오프셋이 하나라도 나오면 실패한다. **차이가 0 이어도 실패한다** — 지금 드리프트가
실재함을 테스트가 스스로 증명하지 못하면 분류는 공허하기 때문이다(#5447 §6-1 의 "조용한 통과"
사고에서 배운 규칙).

## 4. 실측

### 4-1. 드리프트 합계 (rhwp 라이터가 쓴 29건)

```
드리프트 합계 — 색 플래그 58바이트, 할당표 채움 13412바이트
```

색 플래그 58 = 29 × 2. 코퍼스의 중첩 CFB 는 전건 엔트리 4개(Root + `Contents` +
`\x02OlePres000` + `OOXMLChartContents`)라 midpoint BST 의 깊이 1 자식 둘만 red 로 바뀐다.

### 4-2. 독립 오라클 대조

Rust 분류기가 자기 산출을 자기가 채점하지 않도록, 커밋 자산만 읽어 "현재 라이터라면 어느
바이트가 달라지는가" 를 파이썬으로 따로 예측했다(hwpx 변종 14건 — `.hwp` 는 중첩 CFB 가 zlib
스트림 안이라 오라클 범위 밖). 상세 표는
[Stage 1 §2](../../working/archives/task_m100_5967_stage1.md).

| 축 | 파이썬 오라클 (hwpx 14) | Rust 분류기 (29건 중 해당분) |
|---|---:|---:|
| 색 플래그 | 28 (= 14 × 2) | 58 중 28 |
| 할당표 채움 | 6,496 (FAT 5,900 + MiniFAT 596) | 13,412 중 6,496 |

`묶은세로막대형-행추가.hwpx` 는 양쪽 274,432바이트에 **422 = 색 2 + FAT 420** — 이슈 본문의
해부표와 정확히 같다. MiniFAT 버킷도 죽은 가지가 아니다:
`가로막대형_..._단일시리즈제목-점추가`(304B)와 `원형대원형-계열추가`(292B)에서 실제로 걸린다.

### 4-3. red→green 증명

분류기가 헐겁지 않은지 라이터를 일부러 어긋나게 해 확인했다(확인 후 되돌림, `git diff` 공백).

| 변이 | 기대 | 관측 |
|---|---|---|
| FAT 채움 sentinel `FREESECT` → `0xFFFFFFFD` | 버킷 B 분류 실패 → stray | `3차원묶은세로막대형-행추가.hwp: 살림 두 자리 밖 차이 400바이트 (처음 8개 오프셋 [592, 593, ...])` — **FAILED** |
| `write_dir_entry` 가 CLSID 를 0 으로 기록 | 스트림은 통과, CLSID 단언에서 실패 | `3차원묶은세로막대형-대조군.hwpx: 루트 CLSID — 떨구면 한컴이 개체를 잃는다 (#4097)` — **FAILED** |

두 번째는 **대조군**에서 터졌다 — 대조군이 실제 CLSID 를 물고 있고 T1 이 그것까지 본다는 뜻이다.

## 5. 함께 고친 것

| 파일 | 무엇 |
|---|---|
| `samples/issue5447/MANIFEST.json` | `generator` 에 재현 경계 4키 — `byte_reproducible: false`, `byte_reproducibility_lost_at`(원인 커밋), `stream_reproducible: true`, `reproducibility_note`. `counts`·`entries` 무변경 |
| `samples/issue5447/README.md` | "재생성 절차" 의 `sha256sum` 직접 대조 지시를 걷어내고 원인·되돌리지 않는 이유·올바른 대조 축을 적음 |
| `tests/issue_4100_chart_data_edit.rs` | `generate_b2_structure_judgment_bundle` doc comment 한 문단. **코드 변경 없음** |

**판정 산출 38건은 한 바이트도 건드리지 않았다.** `pdf/issue5447/**` 도 무변경이다.

## 6. 검증

| 게이트 | 결과 |
|---|---|
| `regression_suite_025 -- issue_5967` (신규 2건) | 2 passed / 0 failed |
| `regression_suite_015 -- mini_cfb_strict` (라이터 불변식) | 2 passed / 0 failed |
| `issue_4100_chart_data_edit -- b2_judgment_assets_match_the_manifest` | 1 passed / 0 failed |
| `python tools/hancom_chart_judgment_verify.py --rasterizer none` | 통과 — 검사 186건 전건 일치 (38 파일) |
| `python scripts/check_markdown_links.py` | 검사 602건, 깨진 상대 링크 없음 |
| `rustfmt --edition 2021 --check` (변경 `.rs` 2건, LF 정규화 후) | `Diff in` 0건 |
| `cargo clippy --all-targets -- -D warnings` | exit 0, warning 0 |
| `node --test scripts/tests/rust-test-suite-manifest.test.mjs` | 17/18 — 1건은 사전 실패 |

마지막 1건(`CI lint checkout은 PR base 3-way diff를 위해 전체 Git 계보를 가져온다`)은 이 작업과
무관한 **Windows 체크아웃의 CRLF 아티팩트**다. 테스트가 `.github/workflows/ci.yml` 을 `\n`
정규식으로 훑는데 로컬 체크아웃이 `\r\n` 이라 매칭에 실패한다. 이 브랜치는 그 워크플로 파일을
건드리지 않는다.

같은 이유로 `cargo fmt --all -- --check` 는 이 체크아웃 전체에 대해 `Incorrect newline style` 을
낸다(작업 트리가 CRLF). 그래서 변경한 `.rs` 2건만 LF 로 정규화해 `rustfmt --check` 를 돌렸고,
HEAD 판과 작업 판 모두 `Diff in` 0건이다 — 이 변경이 포맷 드리프트를 더하지 않는다.
CI(리눅스, LF 체크아웃)에서는 `cargo fmt --all -- --check` 가 정상 판정한다.

## 7. 알려진 한계

- **분류기는 헤더 DIFAT(109 슬롯)까지만 안다.** DIFAT 섹터를 쓰는 컨테이너(출력 약 7.14MB
  초과)가 판정 자산에 들어오면 조용히 넘기지 않고 소리 내어 실패한다.
- **파이썬 오라클은 hwpx 14건까지다.** `.hwp` 의 중첩 CFB 는 외곽 CFB 안 zlib 스트림이라 순수
  파이썬으로 꺼내지 않았다. `.hwp` 15건은 Rust 분류기 단독 측정이다.
- **`samples/issue5652/` 는 이 브랜치 범위 밖이다.** #5652(PR #5968)가 병합되면 같은 계약을 그
  원장에도 확장할 수 있다 — 그 자산은 애초에 현재 라이터로 만들어졌으므로 드리프트 버킷은
  비어 있을 것이고, `writer_drift_is_confined_to_...` 의 "차이 0 금지" 규칙을 그대로 쓸 수는
  없다. 확장 시 자산별 기대 드리프트 유무를 원장에서 읽어야 한다.

## 8. #5652(PR #5968)와의 관계

무관하다. 원인 코드 `src/serializer/mini_cfb.rs` 는 task5652 diff(97파일)에 없고, task5652 는
`samples/issue5447/`·`pdf/issue5447/` 를 건드리지 않는다. #5652 의 회귀 테스트
`engine_documents_match_spike_documents_except_positional_series_delete` 는 엔진·스파이크 양쪽을
같은 바이너리로 만들어 비교하므로 라이터 판에 무관하다. 두 작업은 순서 의존이 없다.
