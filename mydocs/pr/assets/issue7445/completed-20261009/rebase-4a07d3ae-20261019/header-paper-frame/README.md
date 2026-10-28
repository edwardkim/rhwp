# 헤더 PAPER 사각형의 부모 프레임 복원 (C431)

같은 한컴2020 저장 원본과 독립 Print 전1쪽, 96DPI/compat2022, canonical Native PNG 및 fresh WASM Canvas 출력의 raw2px 실루엣이다. 평균이나 엄격한 픽셀 일치율이 아니다.

- 원본 `tests/fixtures/issue_7642/header-origin/header-paper-shape-hancom-2020.hwp`: `3ae94ff8672e99cf96749db48c0dc2ec78ae993ad708cd423393354d5462607f`
- 독립 Print `tests/fixtures/issue_7642/header-origin/header-paper-shape-print-2020.pdf`: `3f4ec50265bbb47f0c2d85fdde834429dfbc7d30165e22648e6cb522724ecd85`
- fixed upstream `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`: Native/WASM 각각100%
- 수정 전 code `9b1f0e198787ce4d9b70889b8803160d5fa4cdd0`: 양쪽8.70583%
- 수정 후 parent `db00fae605d02764b8d33bd93a0e39063b715156` + runtime2파일: 양쪽100%; patch SHA `979f87a9fe1e3f37bb2bc7bfc057e7c5d75269d9b025ecdd9aa426533228c50b`

Native CLI SHA `da2838e0628f13132702c3d888ec669e5f567bedab44b9b15e48702bfdc5d665`.
fresh WASM SHA `56887cbaec709bb56ac9781e01fe4fea07b668b49da1be264a5bf9cef34ed9fb`.
Mac locked web/pkg/no-opt 대체 빌드이며 Docker 최적화가 아니다.

원본 offset1500/2250HU를 물리 용지(0,0)가 아니라 Header의 부모 프레임에 더한다.
독립 PDF stroke 경계95.468/67.765/195.415/107.723 CSSpx를 ±2px로 보호하며 기존 원본 소유·비율·본문 내용/순서 검사는 유지한다. 같은 새 검사에서 수정 전1FAIL→수정 후1PASS, 정식 Native 관련3건3PASS(exit0)다.

## 대조 범위와 잔여

`comparison.tsv`는 before/after24페이지 쌍이다. Header1쪽×양쪽은 개선했고 대조22쪽은 PNG SHA까지 같다. 0.1%p 이상 하락0쪽. 각 역할/backend의 원본 TSV는 `tsv/`에 둔다.

| 범위 | Native 최저 | WASM 최저 | 전후 |
| --- | ---: | ---: | --- |
| Header 사각형 전1쪽 | 100% | 100% | 양쪽8.70583→100 |
| pic-in-head02 전6쪽 | 57.60580% | 57.60692% | 모두 동일;4쪽 미달 유지 |
| sample11 선택2/50쪽,192DPI | 89.13433% | 90.66313% | 모두 동일;Native50쪽 미달 유지 |
| 본문 Paper A3/A4/Letter 각 전1쪽 | 100% | 100% | 모두 동일 |

대조군의 미달은 이번 헤더 수정 전에도 재현됐다는 뜻이며 upstream 미달이나 해결 완료를 의미하지 않는다. sample11 전151쪽 재검증도 아니다. 엄격한 픽셀 차이·글자 가장자리 차이도 남아 있어 raw100을 완전 픽셀 일치라고 부르지 않는다.

최종 검토에서 사용할 대표는 before-native와 after-native/wasm의 review·overlay6장이다. 전체 회귀 및 실제 Studio 문서 paint/CanvasKit 검증은 별도로 남아 있다. 상세 기록은 `mydocs/working/task_m100_7445_rebase_repair_stage8.md` C431이다. PR 생성·게시 전 사용자 승인을 받는다.
