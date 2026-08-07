---
kind: working
status: active
canonical: mydocs/working/task_m100_4055_stage4.md
last_verified: 2026-08-06
---

# #4055 Stage 4 — 한컴 육안 판정 (S2·S3)

- **Issue**: [#4055](https://github.com/edwardkim/rhwp/issues/4055)
- **판정**: 작업지시자 한컴 오피스(2026-08-05) + HWP 2020 MCP 독립 재현(2026-08-06)
- **결론**: **한컴은 OOXML 표현을 읽는다. 레거시 `Contents` 도 EMF 프리뷰도 안 쓴다.**

## 1. 판정 결과

첫 계열 첫 값을 `4.3` → `91.7` 로 바꾼 변종. 원본 최대값이 `5` 라 반영되면 첫 막대가
차트를 뚫고 솟는다.

| 변종 | ① zip | ② 중첩OOXML | ③ 레거시 | ④ EMF | 한컴 / MCP |
|---|---|---|---|---|---|
| `00-control` (.hwpx / .hwp) | 4.3 | 4.3 | 4.3 | 유지 | 기준 |
| `X-A-zip파트만.hwpx` | **91.7** | 4.3 | 4.3 | 옛 값 | **솟음** |
| `H-A-중첩OOXML만.hwp` | — | **91.7** | 4.3 | 옛 값 | **솟음** |
| `X-B-레거시만.hwpx` | 4.3 | 4.3 | **91.7** | 유지 | 미반영 |
| `H-B-레거시만.hwp` | — | 4.3 | **91.7** | 유지 | 미반영 |
| `X-C-셋다.hwpx` / `H-C-둘다.hwp` | **91.7** | **91.7** | **91.7** | 유지 | 솟음 |
| `X-D-셋다+EMF제거.hwpx` / `H-D-둘다+EMF제거.hwp` | **91.7** | **91.7** | **91.7** | 제거 | 솟음 |

- **S2**: OOXML만 바꾼 X-A·H-A는 반영되고 레거시만 바꾼 X-B·H-B는 미반영이다 → 한컴은
  OOXML을 읽고 레거시 `Contents`는 읽지 않는다.
- **S3**: EMF를 유지한 X-C·H-C와 제거한 X-D·H-D가 모두 반영됐다 → 프리뷰를 쓰지 않고
  다시 그린다. **EMF 재생성 불필요.**

초기 판정은 X-A·H-A로 시작했지만, CLSID 보존 경로를 적용한 뒤 B/C/D와 대조군까지 전수
판정했다. 모든 변종은 오류·복구 대화상자 없이 열렸다.

### HWP 2020 MCP 독립 재현

동일한 10개 입력을 PDF로 변환했다. 모두 `status=success`, `run_status=0`, `validation=ok`,
1페이지 A4였다. 144 DPI 1페이지 렌더 SHA-256은 정확히 두 그룹으로 나뉘었다.

```text
반영   d2effc5d35f5b0ebc5906ed89cb3faf708da6d38d7041df902bd7e569c8c9811
       X-A · X-C · X-D · H-A · H-C · H-D
미반영 6ff074f9e35ef2c67eebee4c8d9cee56e53a7ba527b3403833a567f9fabf3c67
       00-control ×2 · X-B · H-B
```

PDF는 [`pdf/issue_4055_b1_spike/`](../../pdf/issue_4055_b1_spike/)에, 대표 렌더는
[`pr_4061_hancom2020_control.png`](../pr/assets/pr_4061_hancom2020_control.png)와
[`pr_4061_hancom2020_ooxml.png`](../pr/assets/pr_4061_hancom2020_ooxml.png)에 보존했다.

## 2. 도중에 잡은 결함 — `mini_cfb` 가 OLE 루트 CLSID 를 떨군다

첫 판정 시도에서 `H-A`·`H-B` **둘 다 차트가 통째로 비어 그려졌다** — 페이지·눈금자·개체
선택 핸들은 보이는데 차트 내용이 없었다. sentinel 이 반영 안 된 게 아니라 개체 자체가
깨진 것이다.

### 격리

X-A(중첩 CFB 미변경)는 성공했고 H-*(중첩 CFB 재포장)는 전부 실패했다. 원본과 재포장본의
디렉터리 엔트리를 전수 비교했다.

| 필드 | 원본 | 재포장 | 판정 |
|---|---|---|---|
| **루트 CLSID** | `37a13d4c90dcb9479bed59dae352a280` | **전부 0** | ← 유일한 유의미 차이 |
| color | 0 | 1 | 레드블랙 트리 색, 무해 |
| ctime/mtime | — | 현재 시각 | 무해 |
| `OOXMLChartContents` size | 5095 | 5096 | 패치(`4.3`→`91.7`, +1B) |

바깥 HWP CFB 의 루트 CLSID 는 원본도 0 이라 무관하다.

### 원인

`src/serializer/mini_cfb.rs:422,513` 이 CLSID 를 0 으로 고정한다 — 주석까지 있다
(*"CLSID (16바이트 zero) — 이미 0"*). OLE 개체는 **루트 CLSID 로 서버를 식별**한다.
이 코퍼스의 차트는 `{4C3DA137-DC90-47B9-9BED-59DAE352A280}` 를 달고 있고, 비면 한컴이
개체를 알아보지 못해 틀만 그린다. CLSID 를 되박자 정상 렌더됐다.

### 이 결함의 성격

rhwp 는 `parse_ole_container` 가 스트림 **이름**으로 판별하므로 CLSID 손실을 감지하지
못한다. `parse_document` 도, 내가 넣은 조립 검증(`Representations`)도 전부 통과했고
**한컴에서만** 드러났다. #3546 이 지적한 *"rhwp 자신은 자기정합이 유지되므로 자체 왕복
검증으로는 잡히지 않고 한컴 호환만 조용히 깨진다"* 와 같은 계열이다.

`mini_cfb_repack_drops_the_ole_class_id` 로 고정했다. **B1 이 HWP5 를 지원하려면
`mini_cfb` 가 루트 CLSID 를 받도록 선행 수정이 필요하다.**

## 3. 프로브에 반영한 것

- `root_clsid()` / `stamp_root_clsid()` / `rebuild_cfb_preserving_clsid()` 추가
- `mutate_nested()` 가 CLSID 보존 경로를 쓰도록 교체
- S4 테스트가 재포장 후 CLSID 동일성을 함께 단언
- 변종 생성기가 파일 잠금에 관대해짐 — 내용이 같으면 건드리지 않는다(한컴에 파일을 열어둔
  채로 다시 돌릴 수 있게)

## 4. 다음

[최종 보고서](../report/task_m100_4055_report.md)에 S1~S4 답과 B1 본구현 권고를 정리했다.
