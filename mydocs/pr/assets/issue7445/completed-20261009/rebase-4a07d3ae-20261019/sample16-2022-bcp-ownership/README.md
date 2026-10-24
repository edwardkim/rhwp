# 2022 저장본 BCP의 독립 Print 줄 소유 교정 (C432)

원본 `samples/hwp3-sample16-hwp5-2022.hwp` SHA `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`와 독립 한컴2020 Print `pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf` SHA `6737b57af568a591eb073eeb27054e10aaf7587cbdfd6f18a06d1033b4025a19`의 전64쪽을 비교했다.192DPI/compat2022, raw2px 실루엣이다. SVG 변환 PNG가 아니라 NativeSkia 및 fresh WASM Canvas 출력이다.

고정 base `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`, 현재 runtime `d34926b4030df283df075ad0374d445f1dda9820`의 frozen Native47/fresh49를 사용했다. 현재 runtime은 바꾸지 않고 부정확한 기존 회귀를 교정한다.

| 범위 | 전체 최저 | 3쪽 | 90% 미달 |
| --- | ---: | ---: | ---: |
| fixed base Native64쪽 | 59.36766% | 72.37067% | 50쪽 |
| 현재 Native64쪽 | 94.14342% | 99.48430% | 0쪽 |
| 현재 fresh WASM64쪽 | 94.60213% | 99.81839% | 0쪽 |

기존 검사의 한 줄 기대값은 문구 전용 합치기 예외와 같았다. 독립 Print3쪽은 첫 줄 `수`, 둘째 줄 `립`으로 분리한다. 현재 두 줄은 그 내용·원점/기준선·뒤 문단을 보존한다. 예전 단일 줄 검사1FAIL+표 검사1PASS → 독립 경계로 교정한 현재2PASS, 같은 새 검사는 fixed base에서1FAIL+표1PASS다. 정식 Native suite018도2PASS/0FAIL(exit0)다.

완료3쪽은 양쪽98이상과 확대 판독 후 기준선903.20/930.88 CSSpx, 둘째 줄 원점143.36 CSSpx를 독립 PDF pt /0.75에서 ±2px로 보호한다. 쪽·문단 소유·전체 내용·줄 순서/비겹침·후속 문단·표 검사는 함께 유지한다. 페이지별 값은 원본 TSV와 대조 TSV에서 확인한다.

Native63/64쪽은 base99.80954/99.67493→현재97.65514/98.22800%로 각각2.15440/1.44693%p 하락하여 보정이 남았다. 평균 향상으로 상쇄하지 않는다. WASM63/64쪽은99.80727/99.67064%다. 이 증거는 BCP 기대값의 교정 근거이며 PR 준비 완료가 아니다.

현재 각 backend의 canonical TSV와 별도 pdftoppm audit TSV를 구분했다. fixed base WASM은 준비 프로그램 환경이 non-null이고 구 API의 동등성을 확인하지 못해 대조 실행을 수용하지 않았다. 원 driver의 guard 실패는 ignored output에 보존하며 우회하지 않았다. #8064 자동 품질 래칫의 Native-only 범위와 현재 양쪽 시각 수용을 구분한다.

최종 대표는 base/head Native 및 head WASM의3쪽 review/overlay6장이다. 자세한 source/producer·오류 정정·잔여는 stage8 C432를 참조한다. PR 생성·게시 전 사용자 승인을 받는다.
