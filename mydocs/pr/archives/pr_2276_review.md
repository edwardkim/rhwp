# PR #2276 검토 - HWP 생성기 마커 조사 도구

- 검토일: 2026-07-15
- 작성자: planet6897
- 대상: [PR #2276](https://github.com/edwardkim/rhwp/pull/2276), [Issue #2148](https://github.com/edwardkim/rhwp/issues/2148)
- 메타: `devel` 대상, 조사 도구·문서 변경, contributor head `761461c712aa72eda1e415ae151b72ee19960149`
- 검토 범위: PR 본문·대화, 최신 head 소스 diff, 도구 구문·샘플 실행과 원격 CI를 함께 검토했다.
- reviewer: `jangster77` 지정 완료. 최신 head의 원격 CI는 통과 상태다.

## 본문·대화 검토

본문은 동일한 국소 형상에서도 반대 결론이 나오는 [Issue #2148](https://github.com/edwardkim/rhwp/issues/2148)의 판별 신호를 조사하며, HWP5 last-author와 HWPX lastsaveby를
추출하는 도구를 추가한다고 설명한다. 생성기 마커는 부분 판별자일 뿐이고 `1730000`의
plain USER 반례가 남으며, 화이트리스트를 실제 renderer 정책으로 쓰는 변경은 별도 승인이라는
한계가 명시돼 있다.

PR 대화에는 본문을 뒤집거나 renderer 동작을 추가하는 코멘트가 없다. 본문도 소스 렌더링은
바꾸지 않는 조사 산출물임을 일관되게 유지한다.

## 판단

**merge 수용 가능.** `hwp_generator_probe.py`는 동작 경로를 바꾸지 않는 조사 도구이며,
`python3 -m py_compile`과 기존 `samples/hwpx_sample2.hwpx` probe 실행(marker `119473`)을 확인했고
최신 원격 CI도 통과했다. PR에는 전용 HWPX fixture나 자동화 테스트가 포함되지 않았다. 현재 HWPX 추출은 XML parser가 아닌
`lastsaveby` 정규식이므로 attribute 순서·namespace 변형에는 취약하다. 이는 renderer 정책과
분리된 P2 보완으로 남기며, 결과를 별도 승인 없이 화이트리스트나 renderer 동작으로 승격하지 않는다.

## 체리픽 누적 검토 기록

- 순서: 3/3
- 적용 커밋: `761461c712aa72eda1e415ae151b72ee19960149`
- 누적 브랜치 커밋: `6c6c5bf80`
- 충돌: 없음
- 선행 의존: 없음
