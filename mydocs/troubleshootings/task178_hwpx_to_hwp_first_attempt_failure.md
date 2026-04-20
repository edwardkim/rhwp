# #178 첫 시도 실패 — HWPX→HWP 강제 저장으로 사용자 경험 개선 시도

- **일자**: 2026-04-19
- **이슈**: [#178](https://github.com/edwardkim/rhwp/issues/178)
- **브랜치**: `local/task178` (폐기됨, 4587685 의 1커밋만 존재했음)
- **결론**: **시도 실패** — 진짜 원인은 HWPX-IR ↔ HWP-IR 매핑 어댑터 부재

## 1. 시도한 방향

크롬 익스텐션 다운로드 10k 도달, HWPX 저장 깨짐으로 사용자 혼란이 커지는 상황에서:

- `rhwp-studio/src/command/commands/file.ts:save` 와 `rhwp-studio/src/hwpctl/index.ts::SaveAs` 에서
- HWPX 출처여도 항상 `services.wasm.exportHwp()` 호출
- 파일명 `.hwpx` → `.hwp` 자동 정규화

**가정**: HWP 저장 경로는 안정적이므로 (HWP→HWP 라운드트립 검증됨), HWPX 출처도 HWP로 저장하면 한컴 호환 가능.

## 2. 작업지시자 사용자 검증 결과 (실패)

`samples/hwpx/hwpx-h-01.hwpx` (편집 없음 / reflow 보정 후 둘 다):

- **rhwp**: 회색 캔버스, 캐럿만 깜빡임 (페이지 렌더링 안 됨)
- **한컴**: "파일을 읽거나 저장하는데 오류가 있습니다."

## 3. 자동 진단 결과

`tests/hwpx_roundtrip_integration.rs::task178_hwpx_h_01_hwp_export_roundtrip` 으로 측정:

| 시나리오 | rhwp page_count |
|---|---|
| HWP 원본 → HWP 저장 → 재로드 | 1 → 1 (정상) |
| HWPX-h-01 → HWP 저장 → 재로드 | 9 → **209** (23배 폭주) |
| HWPX-02 → HWP 저장 → 재로드 | 6 → **155** (26배 폭주) |
| `reflow_linesegs_on_demand` 강제 후 시도 | 효과 없음 (페이지 수 폭주 동일) |

**TYPESET_VERIFY 경고**:
```
TYPESET_VERIFY: sec0 페이지 수 차이 (paginator=199, typeset=283)
TYPESET_VERIFY: sec1 페이지 수 차이 (paginator=1, typeset=23)
```

페이지네이터와 typeset 결과가 충돌. 모든 문단이 별도 페이지로 인식되는 양상.

## 4. 진짜 원인 (작업지시자 통찰)

> 1. 현재 hwpx 는 rhwp 에서 렌더링이 정상
> 2. hwp 도 IR 로부터 저장시 시리얼라이제이션이 동작
> 3. hwpx 도 IR 에서 그냥 저장하면 당연히 오류
> 4. hwpx 의 경우 저장할 때는 hwp 시리얼라이제이션에 대한 매핑이 필요한건 당연한 것 아닌가?

**HWPX-IR 과 HWP-IR 사이의 매핑 어댑터 부재**.

같은 IR 처럼 보이지만 실제로는 두 포맷이 IR 의 서로 다른 부분 집합·관행을 사용:

- **HWPX 파서**: vpos=0 (사전 계산 안 함), `raw_ctrl_data` 비움, `attr=0`, `Section.raw_stream=None`, `table.common` 사용
- **HWP serializer**: `raw_ctrl_data` 우선, `attr` 비트 연산, `raw_stream` 우선, `table.attr` 사용

**HWPX-IR 을 HWP serializer 에 그대로 넣으면 손상 발생.** 페이지 컨트롤·문단 break_type·표 attr 등 다양한 영역에서 IR 의미 불일치.

기존 문서: `mydocs/tech/hwp_hwpx_ir_differences.md` 에 8개 항목 차이점 정리됨.

## 5. 잘못된 진단 (저자 자신의 오류 기록)

처음에는 lineseg 만 의심했음:
- "한컴이 lineseg 를 textRun 으로 reflow 하니까 rhwp 도 reflow 강제하면 되겠지"

실제 측정 결과 reflow 강제 후에도 페이지 폭주 동일 → lineseg 만의 문제가 아님.

작업지시자 지적으로 진짜 원인 (매핑 부재) 식별. **단일 필드(lineseg) 문제로 좁힌 진단의 잘못**.

## 6. 정리 결정

- `local/task178` 브랜치 폐기 (방법 1, 작업지시자 결정)
- `local/devel` 은 #177 완료 시점 그대로 유지 — HWPX 저장은 기존 깨진 동작 유지 (배포 전과 동일)
- `samples/hwpx/hwpx-h-0[123].hwpx` 디버그 샘플 보존 (작업지시자 직접 커밋 `cc90db4`)
- 본 진단 문서 보존
- #178 이슈 본문에 시도 실패 + 차기 방향 코멘트
- **차기 방향**: HWPX→HWP IR 매핑 어댑터 작업 시작

## 7. 차기 작업 방향

새 타스크에서 HWPX→HWP IR 매핑 어댑터 (가칭 `hwpx_to_hwp_ir_adapter`) 작업:

### 후보 매핑 영역 (현재 알려진 것)

`mydocs/tech/hwp_hwpx_ir_differences.md` 와 본 진단 결과 종합:

1. **페이지 컨트롤**: HWPX 의 `<hp:secPr>` → HWP 의 SECTION_DEF 컨트롤 + raw_ctrl_data 재구성
2. **문단 break_type**: HWPX `pageBreak`/`columnBreak` 속성 → HWP PARA_HEADER break_type 바이트
3. **표 속성**: HWPX `table.common` → HWP `table.attr` 비트 + `raw_ctrl_data` 재구성
4. **셀 속성**: `apply_inner_margin` → list_attr bit 16
5. **lineseg vpos**: HWPX 항상 0 → HWP 사전계산 (reflow 강제 + paginate 결과 기반)
6. **Section.raw_stream 재생성**: HWP serializer 가 raw_stream 우선이라면 HWPX 출처에는 raw_stream 합성 필요
7. **FileHeader / DocProperties**: HWPX 는 정보 부족 → HWP 기본값 채우기

### 진단 다음 단계

1. **단일 변수 격리**: HWPX 로드 → IR 의 어느 필드가 HWP 와 다른지 자동 추출
2. **최소 재현 케이스**: 가장 작은 HWPX (예: ref_text.hwpx) 로 페이지 폭주 재현
3. **단계별 어댑터 적용**: 한 영역씩 매핑 → 페이지 수 회복 확인
4. **검증**: 매핑 적용 후 HWPX → HWP → 재로드 시 페이지 수 = 원본

### 일정 추정

수일 ~ 1~2주 (HWPX 출처 IR 의 차이 영역이 광범위하기 때문).

## 8. 본 실패에서 얻은 교훈

1. **"안정적 경로" 가정의 한계**: HWP serializer 는 HWP 출처 IR 에 한해 안정적이며, **HWPX 출처 IR 은 검증된 적 없음**. 가정 검증 없이 정책 결정은 위험.
2. **단일 필드 진단의 한계**: lineseg 는 차이의 한 단면일 뿐, 매핑 차이는 다층적. 좁은 진단은 잘못된 처방으로 이어짐.
3. **작업지시자의 논리적 검증의 가치**: "당연히 매핑이 필요한 것 아닌가?" 라는 질문이 진단 방향을 정정. 사용자 경험 검증 + 논리적 검증의 결합이 잘못된 진단을 빠르게 잡음.
4. **롤백의 신속함**: 미커밋 변경만 있고 devel merge 전이라 즉시 폐기 가능. 큰 변경은 small 커밋 + 격리 브랜치로 시작하는 것의 가치.

## 9. 관련 자료

- 이슈: [#178](https://github.com/edwardkim/rhwp/issues/178)
- 관련 기존 문서: `mydocs/tech/hwp_hwpx_ir_differences.md` (8개 차이 항목)
- 디버그 샘플: `samples/hwpx/hwpx-h-0[123].hwpx` (cc90db4)
- 폐기된 진단 테스트:
  - `task178_hwpx_h_01_hwp_export_roundtrip`
  - `task178_compare_hwp_native_loadable`
  - `task178_reflow_force_then_export_hwp`
- 폐기된 산출물:
  - `mydocs/plans/task_m100_178.md` (수행계획서)
  - `mydocs/plans/task_m100_178_impl.md` (구현계획서)
  - `mydocs/working/task_m100_178_stage{1..3}.md` (단계 보고서)
  - `mydocs/report/task_m100_178_report.md` (최종 보고서)
- 관련 이슈: #186 (Section 완전 동적화), #185 (rhwp validate CLI)
