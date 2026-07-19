---
kind: reference
status: active
canonical: mydocs/tech/hwpx_hancom_reference.md
last_verified: 2026-07-16
---

# OWPML XML 스키마 reference 자산

`OWPML SCHEMA/`는 HWPX/OWPML 요소와 속성을 확인할 때 사용하는 로컬 XML Schema와 검증
스크립트 묶음이다. 일반 작업 절차가 아니라 reference 자산이므로, 이 폴더의 파일을 변경하거나
새 구현의 유일한 정답으로 사용하지 않는다.

권위 관계:

- HWPX/OWPML 모델의 기술 해설은 [한컴 공식 OWPML 모델 참조 가이드](../tech/hwpx_hancom_reference.md)를 따른다.
- HWP 5.0 바이너리 형식의 구현 정정은 [HWP 5.0 스펙 문서 정오표](../tech/hwp_spec_errata.md)를 따른다.

## 보존 자산

- [Body XML schema](<OWPML SCHEMA/Body XML schema.xml>)
- [Core XML schema](<OWPML SCHEMA/Core XML schema.xml>)
- [Document History XML schema](<OWPML SCHEMA/Document History XML schema.xml>)
- [Header XML schema](<OWPML SCHEMA/Header XML schema.xml>)
- [MasterPage XML schema](<OWPML SCHEMA/MasterPage XML schema.xml>)
- [ParaList XML schema](<OWPML SCHEMA/ParaList XML schema.xml>)
- [Version XML schema](<OWPML SCHEMA/Version XML schema.xml>)
- [PowerShell 검증 스크립트](<OWPML SCHEMA/validate-xsd.ps1>)

## 사용 경계

- 스키마는 XML 요소·속성·자료형을 확인하는 reference다.
- 실제 한컴 동작과 rhwp 구현 판단에는 한컴 원본, 재현 샘플, 기술 정오표를 함께 대조한다.
- 검증 스크립트는 PowerShell 환경에서 실행한다. 다른 운영체제에서는 스키마 자체를 읽거나 동등한 XML
  검증 도구를 사용한다.
