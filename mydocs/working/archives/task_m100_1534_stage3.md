# Stage 3 완료보고서 — Task #1534

> 전수 회귀 검증 + golden 스냅샷 교정

- **이슈**: [#1534](https://github.com/edwardkim/rhwp/issues/1534)
- **브랜치**: `local/task1534`
- **단계**: 3/4 (전수 회귀 검증)
- **작성일**: 2026-06-25

---

## 1. 전체 `cargo test` (--no-fail-fast)

```
ok result 묶음: 148
FAILED 묶음   : 1  → svg_snapshot::form_002_page_0 (golden, 교정 대상)
```

유일 실패는 form-002 SVG golden 1건. 그 외 전 테스트 통과(HWP5 어댑터 #852,
issue_1028/1100/1196/1267/1271/1486/493 등 HWPX 통합 테스트 포함).

## 2. golden 교정 — 결함이 박혀 있던 스냅샷

커밋돼 있던 `tests/golden_svg/form-002/page-0.svg` 에 **이중 이스케이프된 버그
텍스트**가 그대로 캡처돼 있었다(렌더러도 IR 의 `R&amp;&amp;D` 를 화면에 노출).

```diff
-IP R&amp;amp;&amp;amp;D연계        (화면: R&&D 가 아니라 R&amp;&amp;D 로 깨져 보임)
+IP R&amp;&amp;D연계               (화면: R&&D — 한컴과 일치)
-R&amp;amp;&amp;amp;D 자율성트랙(일반)
+R&amp;&amp;D 자율성트랙(일반)
-R&amp;amp;&amp;amp;D 자율성트랙(지정)
+R&amp;&amp;D 자율성트랙(지정)
```

- `UPDATE_GOLDEN=1 cargo test --test svg_snapshot` 로 재생성.
- diff = **3줄(3 insert/3 delete), caption 텍스트 escape 레벨만** 변경. 좌표·폰트·
  기타 요소 변화 없음 → 순수 교정.
- 재생성 후 `svg_snapshot` 8 passed.
- 실패 시 생성된 디버그 산출물 `page-0.actual.svg` 는 제거(추적 대상 아님).

> 이 수정은 **저장(직렬화)뿐 아니라 렌더링 표시도 함께 교정**한다 — 기존에는 폼
> caption 의 `&` 가 화면에도 `&amp;` 로 노출되고 있었다.

## 3. 전수 roundtrip + export-text 재비교 (수정 바이너리)

### batch roundtrip (`samples/hwpx` 전수)

```
총 파일 57 / PASS 56 / IR_DIFF 0 / SERIALIZE_FAIL 0 / REPARSE_FAIL 0 / ROUND2_DIFF 0
PARSE_FAIL 1 = hwpx-01.hwpx (HWP5 OLE 파일이 .hwpx 확장자 — 제외 대상, 결함 아님)
```

### 원본 ↔ 저장본 export-text 일치

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| TEXT-IDENTICAL PASS | 50 | **51** |
| FAIL | form-002, k-water-rfp | **k-water-rfp (별건)** |

- **form-002 PASS 전환** — 본 결함 해소 확인.
- 나머지 50건 **무회귀**.
- `k-water-rfp` 는 페이지 경계 빈 줄 증가(문단 구조·본문 보존)로 본 이슈와 무관한
  별건(이슈 #1534 본문에 별건 명시). 범위 외.

## 4. 정적 검사

- `cargo clippy --release` : 신규 경고/에러 **0**.
- 빌드 경고 **0**.

## 5. 리스크 판정

A안(전역 `attr_str` unescape) 채택 결과 회귀는 golden 1건(교정)뿐이며, B안 폴백
불필요. 전역 변경의 blast radius 가 자기교정적임을 전수 검증으로 확인.

## 6. 산출물 / 커밋

- `tests/golden_svg/form-002/page-0.svg` (교정)
- `mydocs/working/task_m100_1534_stage3.md` (본 보고서)

## 7. 다음 단계

Stage 4 — 최종 결과보고서 작성.
