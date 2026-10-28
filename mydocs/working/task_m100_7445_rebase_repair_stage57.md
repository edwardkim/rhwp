# #7445 리베이스 보정 57단계: 과거 PR 래칫의 자동 연결 제거

## 요청과 변경 범위

사용자는 #8148 강제 검사를 제거하도록 요청했고, 실제 연결이 #8064 Native 래칫뿐임을 확인한 뒤
“#8064를 포함한 과거 PR 품질 래칫의 자동 강제 연결도 제거”라고 범위를 확정했다.
변경 전 HEAD는 `c639030cf0a121f1c50a697c9733f5572d6fce31`, 고정 base는
`605ccfd92f2f12f18e14182548d729f9929fda00`이다.

`visual_sweep.py`의 일반 보고서와 실루엣 전용 경로에서 `enforce_quality` 호출 및 import를
제거했다. 과거 PR 계약을 기본 실행에서 자동으로 읽거나 gate에 결합하지 않는다.
독립적으로 호출하는 비교 helper·계약 JSON·과거 TSV는 보존하고 설명을 명시적 비교 용도로
정정했다. raw 2px 90%·전체 쪽수·측정 불가 검사와 출력 provenance는 유지한다.
실루엣만 산출한 실행은 직접 판독을 대신하지 않으므로 `not_evaluated`를 유지한다.

## 실제 변경 전후 대조

동일한 frozen Native 바이너리, 기존 sample16 HWP5 원본·독립 Print PDF·글꼴 공급으로
18·35쪽을 일반 보고서와 실루엣 전용 경로에서 각각 재실행했다. 측정 조건은 96DPI,
Print, compat 2024, 캡처 총 8이다. 렌더러 구현·PDF rasterizer·허용치는 바꾸지 않았다.
이 두 쪽은 연결 제거 검증이며 전체 문서나 PR 수용 결과가 아니다.

| 경로 | 변경 전 | 변경 후 | PNG 및 측정값 |
| --- | --- | --- | --- |
| 일반 보고서 | 과거 PR 래칫으로 `re_review_required` | 기본 gate `passed` | PNG SHA 및 경로를 제외한 모든 지표 동일 |
| 실루엣 전용 | 과거 PR 래칫으로 `re_review_required` | `not_evaluated` | PNG SHA 및 TSV 동일 |

두 쪽 raw 값은 98.49531%, 98.01124%로 동일하다. 전체 쪽수 메타데이터도 양쪽 64/64다.
최초 비교 스크립트는 실루엣 전용 상태를 `passed`로 잘못 기대했고, 다음 비교에서는 서로 다른
출력 디렉터리 문자열까지 같아야 한다고 검사했다. 두 진단 assertion을 정정한 후 상태·PNG SHA·
실제 지표 동일성을 확인했다. 재캡처하거나 측정값을 수정한 결과가 아니다.

기존 `test_visual_sweep*.py` 전체 189건이 7.868초에 통과했다. 여기에는 raw 미달을
보정값으로 구제하지 않는 검사, 정확히 90%의 통과, 측정 불가·쪽수 불일치의 보류 검사가 포함된다.
`git diff --check`도 통과했다. 렌더러 소스를 바꾸지 않은 이 단계에서 Cargo 전체 검증을
중복 실행하지 않았다. 앞 단계의 미완료 전체 회귀·시각·실제 소비 검증을 통과로 간주하지 않는다.

로그·producer·변경 전후 결과·`comparison.json`은 ignored
`output/pr-review/7445-completed-20261009/rebase-8155-20261022/visual-quality-unlink-20261023/` 및
같은 부모의 `visual-quality-unlink-20261023-{before,after,unittest}.log`에 보존한다.
