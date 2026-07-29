# PR #3537 검토 기록 — HWPX 참조·여백 왕복 보존

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3537](https://github.com/edwardkim/rhwp/pull/3537) — `secPr`·BinData·ParaShape 무손실 (#2779, #3526, #3368) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commits | `c05c4a51b4893d29c7e7958385d5bb00c7306de2` / `8949869af`, `56c4af4b6`, `52bcc4b9d` |

세 고유 commit은 `secPr.memoShapeIDRef`와 각주/미주의 placement 매핑, stream이 없는 embedding
BinData의 manifest 등록, 홀수 HWPUNIT ParaShape 여백의 default 원값 보존을 다룬다. 그림의 bytes가
없어도 manifest index를 지켜야 참조 그림·앵커가 통째로 빠지지 않으며, `case/default`의 ±1 관계에서만
원값을 택해 기존 동일값 패턴을 바꾸지 않는다.

## 검증과 판정

- #3534와 파일은 일부 인접하지만 변경 의미가 달라, #3534 다음에 #3537을 적용해 충돌 없이 두 범위를
  보존했다.
- HWPX serialize/parse·IR fixture 회귀와 release library·전체 test gate를 통합 실행한다.
- contentless BinData의 빈 href 등 패키지 기하 보존 경로는 입력 manifest 유지가 우선인 의도된
  동작이며, 새 stream 생성 기능으로 범위를 확장하지 않는다.

명시적 parser/serializer 쌍의 무손실 규약을 강화하므로 **기술적 수용 가능**이다.
