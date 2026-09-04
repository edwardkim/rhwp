# task_m100_5249 처리결과 — `secd` CTRL_HEADER tail 을 저장 버전으로 확정

- 이슈: [#5249](https://github.com/edwardkim/rhwp/issues/5249) · 원인 조사 원천 [#2768](https://github.com/edwardkim/rhwp/issues/2768)
- 분석: [`mydocs/working/task_m100_5249_stage1.md`](../../working/task_m100_5249_stage1.md)
- 기준: `0697bc559` (devel)

## 1. 확정한 계약

**`secd` CTRL_HEADER 확장 tail 의 길이는 저장될 파일 형식 버전이 정한다.**

| 저장 버전 | 확장 tail | CTRL_HEADER | 한컴 저작 구역 |
|---|---|---|---:|
| 5.0.1.7 (관측 하한) | 8 byte | 36 | 4 |
| 5.0.2.4 ~ 5.0.3.4 | 10 byte | 38 | 188 |
| **5.0.4.0 이상** | **19 byte** | **47** | **325** |

한컴 저작 517구역에서 예외 0. 재현은 `python scripts/secd_tail_survey.py`.

## 2. #2768 의 두 기존 계약 처리 (수용 기준 7)

| 계약 | 처리 | 근거 |
|---|---|---|
| 어댑터 바탕쪽 게이트 (`master_pages.is_empty()`) | **폐기** | 양방향 반증 — 바탕쪽 0인데 47이 **284구역**, 바탕쪽 있는데 38이 **10구역** |
| 직렬화기 #1058 "한컴 정답지 = 38" | **범위를 명시해 유지** | 5.0.4.0 **미만**에서만 참(188/188). `raw_ctrl_extra` 가 빈 기존 기본 경로는 그대로 |
| (신설) 버전 게이트 | **채택** | 517/517, 예외 0 |

#1058 의 정답지는 5.0.3.x 시절 파일이었을 것이다 — 그때는 옳았고, 파일 형식이 5.0.4.0 에서
확장 영역을 8 → 17 byte 로 늘리며 어긋났다.

## 3. 무엇이 잘못돼 있었나

HWPX 파서는 FileHeader 에 **5.1.0.0** 을 적는다. 그런데 tail 은 바탕쪽이 있을 때만 19 byte 로
합성했으므로, 바탕쪽 없는 HWPX 를 변환하면 **버전은 47을 약속하고 내용은 38을 내보내는**
문서가 나왔다. 코퍼스에 커밋돼 있던 rhwp 산출물 3건이 정확히 그 상태다(`RhwpHwpxOrigin`
스트림 보유, 5.1.0.0 인데 secd 38) — 반례가 아니라 **결함의 증거**였다.

## 4. 변경

| 파일 | 내용 |
|---|---|
| `src/document_core/converters/hwpx_to_hwp.rs` | `section_def_ctrl_tail_len()` 신설(버전 → 길이), `materialize_section_def_master_page_tail` → `materialize_section_def_ctrl_tail` 로 대체. 바탕쪽 게이트 제거, `raw_ctrl_extra` 보존 가드 추가 |
| 〃 (호출부) | `adapt_section_def` 가 확정된 FileHeader 버전을 받는다 (`normalize_file_header_for_hwp` 뒤라 값이 이미 최종) |
| `tests/cases/issue_5249_section_def_ctrl_tail.rs` (신규) | 변환 산출 바이트에서 secd 를 뜯어 길이·tail 을 판정. 한컴 정답지 대조 포함 |
| `scripts/secd_tail_survey.py` (신규) | 코퍼스 전수 진단. 의존성 0(CFB 리더 자체 구현), 출처(`RhwpHwpxOrigin`) 분리 |

적용 범위는 **HWPX→HWP 합성 경로뿐**이다(이슈 목표 4). HWP5 원본에서 파싱한
`raw_ctrl_extra` 는 손대지 않는다(목표 5) — 그 가드를 코드로 못박았다.

## 5. 검증

### red → green (수용 기준 3)

가로채기 대신 종전 게이트(`master_pages.is_empty()` 조기 반환 + 고정 19)를 임시로 되살려 돌렸다.

```
test issue_5249_section_def_ctrl_tail::hwpx_without_master_pages_still_gets_the_versioned_tail ... FAILED
    assertion `left == right` failed: 5.1.0.0 저장본의 secd 는 47 byte 여야 한다
    (바탕쪽 유무와 무관): 38 byte
```

되돌리면 2/2 통과. 정답지 대조 테스트는 그 문서에 바탕쪽이 있어 종전 게이트에서도 통과한다 —
**회귀 고정용**이지 red→green 증거가 아니다(정직하게 적는다).

### 한컴 정답지 대조 (수용 기준 5)

`samples/hwpx/exam_social-p1.hwpx` 를 변환한 산출과 같은 문서의 한컴 저작본
`samples/exam_social-p1.hwp`(5.1.0.1):

| | secd | tail |
|---|---:|---|
| 한컴 정답지 | 47 | `00…00` (19 byte 전부 0) |
| rhwp 변환 (수정 후) | 47 | `00…00` — **바이트 동일** |

버전이 다르면 tail 도 달라야 한다는 것도 같이 확인된다 —
`143E433F503322BD33` 은 한컴이 5.0.3.2 로 저장해 38 이고, rhwp 는 5.1.0.0 으로 저장하므로 47 이다.
둘 다 각자의 버전 계약을 지킨다.

### 실행한 것

| 명령 | 결과 |
|---|---|
| `cargo test --lib -p rhwp` | **3,895 passed** / 13 ignored |
| `cargo test --test regression_suite_009` (신규 계약 포함) | 132 passed |
| `cargo test --test regression_suite_001` (`hwpx_to_hwp_adapter`) | 74 passed / 15 ignored |
| `cargo test --test regression_suite_028` (`convert_verify_corpus_ratchet`) | 126 passed / 2 ignored |
| `cargo test --test regression_suite_007` (HWP3→HWP 변환 계약) | 133 passed |
| `rustfmt --edition 2021` (변경 파일 per-file) | 차이 없음 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `python scripts/secd_tail_survey.py` | 517구역 집계 재현 |

## 6. 남긴 것 (수용 기준 2 의 잔여)

- **확장 영역의 비영 u16** — 47 byte tail 325구역 중 14구역이 offset 0 또는 4 에 1·2·4 를 갖는다.
  바탕쪽 수의 함수가 아니다(바탕쪽 1인데 1·4·0 이 공존). 결정 요인 미확정이므로 종전
  정답지(exam 계열)에서 유도된 규칙을 **넓히지도 좁히지도 않고** 그대로 뒀다. 다만 그 규칙이
  10 byte tail 을 침범하지 않도록 확장 tail 로 범위를 좁혔다(코퍼스 38 byte 188구역 전부
  그 자리가 0).
- **버전 경계 미관측 구간** (5.0.3.4, 5.0.4.0) — 코퍼스에 없다. 보수적으로 구 계약으로 떨어진다.
- **36 byte(5.0.1.7)** — 관측만 했다. HWPX 출처는 그 버전으로 저장되지 않아 합성 대상이 아니다.
