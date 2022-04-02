# IPWIS

## Procedure

1. 여러 공급자가 런타임을 실행하고, 네트워크에 연결합니다.
1. 사용자는 본인이 서명한 AVUSEN Metadata 호환 함수 템플릿을, 후불 보상을 제안함과 동시에 한 공급자에 제출하여 실행을 요청합니다.
1. 공급자는 함수가 공급자의 자원으로 실행 가능한지, 보상이 적절한지, 그리고 사용자로부터 추후 보상을 충분히 받을 수 있는지 검증합니다.
1. 공급자는 함수를 실행하고, 결과를 사용자 및 다음 함수가 복호화할 수 있는 형태로 제공합니다.
1. 결과는 다음 함수로 전달되며, 최종적으로 사용자에게 보고됩니다.

## License

* IPWIS Modules (`ipwis-modules-*`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
* IPWIS Runtime (`/runtime/*` / `ipwis-runtime`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

The reason for the split-licensing is to ensure that for the vast majority of teams using IPWIS to create feature-chains, then all changes can be made entirely in Apache2-licensed code, allowing teams full freedom over what and how they release and giving licensing clarity to commercial teams.
