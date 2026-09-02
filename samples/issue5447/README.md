# samples/issue5447 — #5447 B2 구조 편집 한컴 판정 입력

**이 폴더는 일반 fixture 가 아니다.** [#5447](https://github.com/edwardkim/rhwp/issues/5447)
B2 스파이크에서 **한컴으로 판정한 38개 산출**이다. 현행 기준 PDF 는 원본의 저장 제품 메타데이터로
engine을 결정한 [`pdf/issue5447/`](../../pdf/issue5447/)의 `-2020.pdf` 또는 `-2024.pdf`이며,
판정 원장은 이 폴더의
[`MANIFEST.json`](MANIFEST.json), 결론은
[`mydocs/report/task_m100_5447_report.md`](../../mydocs/report/task_m100_5447_report.md) 다.

## 회귀 코퍼스로 승격하지 말 것

이 파일들은 **일부러 낡고, 일부러 깨져 있다.**

- **`c:f` 참조 범위를 갱신하지 않았다.** 행을 늘려도 `c:f` 는 옛 4행 범위 그대로다 —
  한컴이 낡은 참조를 어떻게 다루는지가 판정 대상이었기 때문이다(#5447 §3-1).
- **③ 레거시 `Contents` 와 ④ EMF 프리뷰도 갱신하지 않았다.** 편집 전 바이트 그대로다.
- **두 건은 의미가 깨진 경계 변종이다** — 이것이 이 파일들의 존재 이유다:
  - `원형대원형-계열추가` — ofPie 에 2번째 계열. 한컴이 조용히 무시한다
  - `시가고가저가종가-계열삭제` — 주식형 4→3 계열. `c:upDownBars` 캔들 장치가 남아
    고가·저가를 몸통으로 그린다

"정상 문서" 를 기대하는 코퍼스 스윕에 넣으면 잘못된 신호를 낸다. `samples/` 를 훑는 기존
게이트는 전부 비재귀이고 `samples/chart` 만 명시적으로 합류하므로
(`tests/convert_verify_corpus_ratchet.rs`) 이 폴더는 어디에도 잡히지 않는다. 그 상태를 유지한다.

## 구성 — 38 파일

| 역할 | 수 | 무엇 |
|---|---|---|
| 대조군 | 9 `.hwpx` | `samples/chart/**` 원본 무편집 사본. 변종이 쓰는 기준 문서마다 하나 |
| 변종 | 14 × 2포맷 = 28 | 행·점 삽입/삭제, 계열 복제/삭제, 계열명·카테고리 라벨 교체 |
| 변환본 | 1 `.hwp` | `묶은세로막대형-행추가` 를 HWPX 에서 만들어 HWP5 로 변환 (①이 ②로 접힌다) |

`PANJEONG.md` 는 이 꾸러미와 함께 작업지시자에게 보낸 **판정 지시표**다 — 무엇을 어떻게 봐 달라고
요청했는지가 판정의 일부이므로 산출물과 같이 보존한다.

## 원장과 재계산

`MANIFEST.json` 이 38행 각각에 대해 원본 경로·SHA-256, 원본 메타데이터로 확인한 engine의 현행
한컴 PDF 경로·SHA-256, 144dpi 래스터 해시 2축, 판정, 편집기 관측을 적어 둔다. **파일별 전체
SHA-256 은 이 원장이 유일한 정본**이고 README 에 복사하지 않는다 — 두 곳에 적으면 한 곳이 조용히
늙는다. 같은 디렉터리의 `-2022.pdf`는 이전 판정 PDF로 보관하되 현행 원장 기준에는 속하지 않는다.

```bash
# 원장 전체 재계산 — 파일 SHA-256 + 래스터 + 판정 + 불변식
python tools/hancom_chart_judgment_verify.py

# 보고서 §2 의 해시 축(poppler)으로 재계산
python tools/hancom_chart_judgment_verify.py --rasterizer pdftoppm
```

`tests/issue_4100_chart_data_edit.rs::b2_judgment_assets_match_the_manifest` 가 같은 대조의
파일 해시 부분을 CI 에서 상시로 돌린다(렌더러 의존성이 없어서다).

## 재생성 절차

산출을 다시 만들려면:

```bash
cargo test --profile release-test --test issue_4100_chart_data_edit \
    generate_b2_structure_judgment_bundle -- --ignored --nocapture
```

생성기는 gitignored `output/issue_5447_b2_judgment/` 에 쓴다. **어긋나더라도 이 폴더가
정본이다** — 한컴이 실제로 연 파일은 여기 있는 바이트이고, 판정은 그 바이트에 대한 관측이기
때문이다. 재생성 결정성 실측은
[`mydocs/working/task_m100_5447_stage2.md`](../../mydocs/working/task_m100_5447_stage2.md) 에 있다.

### 바이트로 대조하지 말 것 — 스트림으로 대조한다 (#5967)

**재생성본은 이 폴더와 바이트가 반드시 어긋난다.** devel `b9eb55107`("검토: CI 완료 외부 PR
7건 통합 수용 (#5912)", 외부 PR cherry-pick `97dbf4cef`·`6a0c04159`)이 `mini_cfb` 를 MS-CFB
쪽으로 고쳤기 때문이다 — 디렉터리 red/black 을 실제로 칠하고(`color_deepest_nodes`),
FAT·MiniFAT 미사용 슬롯을 `0x00` 대신 FREESECT 로 선채운다. 차이는 중첩 CFB 의 컨테이너 살림
두 자리에만 있고 **차트 스트림은 동일하다**. 예: `묶은세로막대형-행추가.hwpx` 는 양쪽 274,436
바이트에 422바이트 차이 — 색 플래그 2 + FAT 꼬리 미사용 슬롯 105개 × 4바이트.

되돌리지 않는다. 종전 "전 엔트리 black" 은 leaf 깊이가 갈릴 때 black-height 가 어긋나고
미사용 슬롯 `0x00` 은 strict 파서에서 sector 0 참조로 읽힌다.
[`tests/cases/mini_cfb_strict_contract.rs`](../../tests/cases/mini_cfb_strict_contract.rs) 가
그 반대 방향을 이미 상시로 고정한다. 유효성 문제도 없다 — 현재 라이터 포장 상태의 산출을
한글 2022 가 전건 정상 개봉·렌더했다(#5652 판정, 개봉 실패 0).

그래서 재생성 대조 축은 **중첩 CFB 의 스트림 동일성**이다. 그 계약은
[`tests/cases/issue_5967_cfb_repack_reproducibility.rs`](../../tests/cases/issue_5967_cfb_repack_reproducibility.rs)
가 CI 에서 상시로 지킨다 — 스트림 이름·바이트·루트 CLSID 가 재조립을 넘어 보존되는지(38건),
그리고 바이트 차이가 색 플래그와 할당표 미사용 슬롯 **밖으로 새지 않는지**(29건). 손으로
`sha256sum` 을 대조하지 말고 이 테스트를 돌린다.
