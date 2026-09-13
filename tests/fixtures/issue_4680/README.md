# issue_4680 실물 검증 입력

- 원본 이름: `1170000-200500003_D0150004-1-001_독일의 법령체계와 입법심사기준(최종본).hwp`
- 출처: https://github.com/edwardkim/rhwp/issues/4680에 식별된 공개 정부 문서. 메인터너가 보관한 `korea_downloads` 원본과 동일하다.
- 저장소 경로: `tests/fixtures/issue_4680/german-legislative-system.hwp`
- 크기: 545110 bytes
- SHA-256: `543d67cdb4d84b876949cef4f4ec7435b99716d6fa7feebde57b024c88d40559`
- 역할: 통합 검토의 실제 입력. 합성 테스트와 구분하며, 파일 존재만으로 한컴 호환성을 주장하지 않는다.

## 검토에서 사용한 파일

| 파일 | bytes | SHA-256 |
| --- | ---: | --- |
| `german-legislative-system-before.hwp` | 369152 | `c3926c32c65f29968b7134e175d176b99f2bce37f7205057cf2df16d11e72679` |
| `german-legislative-system-candidate.hwp` | 369152 | `78d1bf6cc4480619d2044466c6f3742fc2bac7da4b2b6cb9abbdfc158ec63735` |
| `german-legislative-system-hancom-2020.hwp` | 538112 | `84bcec55e53692a935dafec0ff509f878b278c9f833e59a3f4c9be1a444444b6` |
| `german-legislative-system.hwp` | 545110 | `543d67cdb4d84b876949cef4f4ec7435b99716d6fa7feebde57b024c88d40559` |

`before`와 `candidate`는 각각 devel `897c6a3d8`와 통합 code `4aa80b96a`의 `rhwp convert` 산출이다.
`hancom-2020`은 원본을 한컴 MCP engine 2020으로 저장한 독립 기준(HWP5)이며 job `da6841d8-6e2b-4f43-aa11-71848ca0cf3f`다.
후보 저장본의 한컴 개방 검증 PDF는 `pdf/german-legislative-system-candidate-2020.pdf`(326쪽)에 보존한다.
원본과의 페이지 일치 증거가 아니라 저장본을 열어 출력할 수 있다는 증거다. #4680 전체 완료를 주장하지 않는다.
