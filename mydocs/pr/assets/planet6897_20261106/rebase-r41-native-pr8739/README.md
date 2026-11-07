# PR #8739의 R41 Native 기능 보존

리베이스로 들어온 PR #8739의 독립 Print 9개 입력·24쪽을 새 devel과 후보에서 실제 비교했다. 새로 체리픽한 PR이 아니다.

- 새 base: `e4ffabea38062e716f6c333cc950fa31b42b4dd7`
- 후보 제품·검증 source: `851e4074514935e4c6a7377f71a97f0cca454191`
- base Native SHA-256: `2c1e0a8ce639a6fda287b599806aace25602686658b884c1f5967559d1a188eb`
- 후보 Native SHA-256: `45f9b5a2fe5bf0a4eede663b22841146f496d58cb02cfd3f9fbf069104a3b222`
- Print profile·compat 2022·96 DPI·같은 입력·PDF·전체 시스템 글꼴로 실행했다.
- 24쪽 모두 base/후보 PNG가 바이트 단위로 같다. 0.1%p 초과 하락·독립 PDF 쪽수 불일치·90% 미만은 없다.
- [입력·PDF·TSV SHA 및 전쪽 결과](inventory.tsv), [각 페이지 비교](native-page-comparison.tsv).

| 문서 | 전체 쪽수 | base 최저 raw | 후보 최저 raw | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| `radial` | 6 | 100.00% | 100.00% | [base](base-e4/radial.tsv) · [후보](candidate/radial.tsv) |
| `eq-01` | 1 | 95.43% | 95.43% | [base](base-e4/eq-01.tsv) · [후보](candidate/eq-01.tsv) |
| `shape-group` | 1 | 100.00% | 100.00% | [base](base-e4/shape-group.tsv) · [후보](candidate/shape-group.tsv) |
| `picture-crop` | 1 | 100.00% | 100.00% | [base](base-e4/picture-crop.tsv) · [후보](candidate/picture-crop.tsv) |
| `leading-definition` | 1 | 100.00% | 100.00% | [base](base-e4/leading-definition.tsv) · [후보](candidate/leading-definition.tsv) |
| `recent-8555` | 4 | 99.92% | 99.92% | [base](base-e4/recent-8555.tsv) · [후보](candidate/recent-8555.tsv) |
| `recent-8556` | 5 | 99.93% | 99.93% | [base](base-e4/recent-8556.tsv) · [후보](candidate/recent-8556.tsv) |
| `recent-8557` | 4 | 99.92% | 99.92% | [base](base-e4/recent-8557.tsv) · [후보](candidate/recent-8557.tsv) |
| `issue8337-full-fonts` | 1 | 99.74% | 99.74% | [base](base-e4/issue8337-full-fonts.tsv) · [후보](candidate/issue8337-full-fonts.tsv) |

## 실제 원 수식 face 공급과 부재 경계

같은 공개 family를 두 프로그램으로 나눠 공급한 검사는 base에서 실제 독립 윤곽 assertion으로 실패했다. 후보에서는 이 검사와 원 글꼴 완전/부분 공급·부재, 명시적 대체, 실제 producer의 private 숫자 4·3을 포함한 7건이 모두 통과했다. Native와 Print의 점수만으로 숫자의 의미나 원 프로그램 선택을 대신하지 않는다.

이 문서는 완료한 Native 범위다. 최적화 fresh WASM·나머지 원 PR의 전쪽 검증·편람 gate·최종 전체 회귀와 lint를 완료한 것으로 표시하지 않는다.
