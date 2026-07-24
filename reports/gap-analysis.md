# EL10 Gap Analysis Report

## Executive Summary

| Metric | Count |
|--------|-------|
| Spec files analyzed | 869 |
| Total packages (including subpackages) | 1367 |
| Total Requires | 2484 |
| Total BuildRequires | 2716 |
| Satisfied Requires | 2394 (96.4%) |
| Satisfied BuildRequires | 2665 (98.1%) |
| **Missing dependencies** | **53** |
| Version mismatches | 17 |
| Unresolved macros | 74 |

### Repos Analyzed

- **foreman-packaging** (`foreman-packaging`): 586 specs, 1046 packages
- **pulpcore-packaging** (`pulpcore-packaging`): 283 specs, 321 packages

### EL10 Repos Queried

- BaseOS
- AppStream
- CRB

## Runtime Version Matrix

| Runtime | EL10 Version | Source | Required Versions |
|---------|-------------|--------|-------------------|
| Ruby | 3.3.10 | AppStream | < 3.1, < 3.4, < 4, < 4.0, < 5, > 1.9.3, >= 1.8.1, >= 1.8.7, >= 1.9.0, >= 1.9.1, >= 1.9.2, >= 1.9.3, >= 2.0, >= 2.0.0, >= 2.1, >= 2.1.0, >= 2.2, >= 2.2.0, >= 2.3, >= 2.3.0, >= 2.4, >= 2.4.0, >= 2.4.1, >= 2.5, >= 2.5.0, >= 2.6, >= 2.6.0, >= 2.6.7, >= 2.7, >= 2.7.0, >= 3.0, >= 3.0.0 |
| Python | 3.12.13 | BaseOS | - |
| Node.js | 22.23.1 | AppStream | >= 22 |
| PostgreSQL | 16.14 | AppStream | - |
| Rails | 7.0.10 | foreman-packaging | < 7.1.0, = 7.0.10-1.el10, >= 7.0.3 |
| Django | 4.2.30 | pulpcore-packaging | < 5.0.0, < 5.3, >= 2.0, >= 2.2, >= 3.0, >= 3.1.1, >= 3.2, >= 4.2, >= 4.2.24 |
| Ansible | 2.16.19 | AppStream | = 1:2.16.14-3.el10, >= 1:2.16.14-3 |
| Puppet/OpenVox | N/A | - | >= 7, >= 8.0.0, >= 8.23.1 |
| systemd | 257 | BaseOS | - |
| OpenSSL | 3.5.7 | BaseOS | - |

## Missing Dependencies

Dependencies not found in EL10 repos or self-provided by packaging repos.

| Dependency | Type | Virtual | Required By (count) | Example Packages |
|-----------|------|---------|--------------------|-----------------|
| `python3.11-setuptools` | BuildRequires | no | 17 | python-click-shell, python-colorama, python-commonmark (+14) |
| `python3.11-devel` | BuildRequires | no | 14 | python-click-shell, python-colorama, python-commonmark (+11) |
| `python3.12-rpm-macros` | BuildRequires | no | 7 | ansible-core, ansible-runner, python-jinja2 (+4) |
| `openvox-agent` | Requires | no | 6 | foreman-installer, foreman-installer, puppet-agent-oauth (+3) |
| `ruby(abi)` | Requires | yes | 4 | rubygem-bundler_ext, rubygem-bundler_ext, rubygem-concurrent-ruby-edge (+1) |
| `python3.11-requests` | Requires | no | 3 | ansible-collection-theforeman-foreman, ansible-collection-redhat-satellite, python3.11-pyjwkest |
| `rh-python38-scldevel` | BuildRequires | no | 3 | %scl_name, %scl_name-build, %scl_name-scldevel |
| `python2-devel` | BuildRequires | no | 2 | katello-host-tools, python2-libcomps |
| `libsass.so.1()(64bit)` | Requires | yes | 2 | rubygem-sassc, rubygem-sassc |
| `rh-python38-python-devel` | BuildRequires | no | 2 | %scl_name, %scl_name-python3-devel |
| `candlepin-selinux` | Requires | no | 2 | katello, katello-selinux |
| `python3.11-pyyaml` | Requires | no | 2 | ansible-collection-theforeman-foreman, ansible-collection-redhat-satellite |
| `python3.11-social-auth-app-django` | Requires | no | 2 | python3.11-galaxy-ng, python3.11-galaxy-ng |
| `python3.11-social-auth-core` | Requires | no | 2 | python3.11-galaxy-ng, python3.11-galaxy-ng |
| `python3.11-pulpcore` | Requires | no | 2 | python3.11-galaxy-ng, python3.11-galaxy-ng |
| `python3.11-pulp-ansible` | Requires | no | 2 | python3.11-galaxy-ng, python3.11-galaxy-ng |
| `python3.11-pulp-container` | Requires | no | 2 | python3.11-galaxy-ng, python3.11-galaxy-ng |
| `libopenscap.so.25()(64bit)` | Requires | yes | 1 | rubygem-openscap |
| `python2-tracer` | Requires | no | 1 | katello-host-tools-tracer |
| `satellite-lifecycle` | Requires | no | 1 | rubygem-foreman_theme_satellite |
| `zchunk` | BuildRequires | no | 1 | createrepo_c |
| `python3.11-dynaconf` | Requires | no | 1 | python3.11-galaxy-ng |
| `salt-master` | Requires | no | 1 | rubygem-smart_proxy_salt |
| `python3.11-wheel` | BuildRequires | no | 1 | python-galaxy-ng |
| `python3.11-idna` | Requires | no | 1 | python3.11-idna-ssl |
| `python3.11-django-automated-logging` | Requires | no | 1 | python3.11-galaxy-ng |
| `python3.12-opentelemetry_proto == 1.40.0` | Requires | no | 1 | python3.12-opentelemetry_exporter_otlp_proto_common |
| `candlepin` | Requires | no | 1 | katello |
| `python3.12-pulp-glue == 0.39.1` | Requires | no | 1 | python3.12-pulp-cli |
| `python3.12-pulp-glue-deb == 0.4.4` | Requires | no | 1 | python3.12-pulp-cli-deb |
| `python3.12-sigstore-models == 0.0.6` | Requires | no | 1 | python3.12-sigstore |
| `foreman_theme_satellite_assets` | BuildRequires | no | 1 | rubygem-foreman_theme_satellite |
| `rh-python38-runtime` | Requires | no | 1 | %scl_name-runtime |
| `(puppet-agent >= 7 with puppet-agent < 9)` | Requires | yes | 1 | puppetlabs-stdlib |
| `rh-python38-python-test` | Requires | no | 1 | %scl_name-python3-test |
| `rh-python38-python-wheel` | Requires | no | 1 | %scl_name-python3-wheel |
| `python3.11-django-prometheus` | Requires | no | 1 | python3.11-galaxy-ng |
| `python3.11-click` | Requires | no | 1 | python3.11-click-shell |
| `libopenscap.so.25` | Requires | no | 1 | rubygem-openscap |
| `rh-python38-python-setuptools` | Requires | no | 1 | %scl_name-python3-setuptools |
| `python3.12-sigstore-rekor-types == 0.0.18` | Requires | no | 1 | python3.12-sigstore |
| `python3.11-galaxy-importer` | Requires | no | 1 | python3.11-galaxy-ng |
| `python3.12-opentelemetry_api == 1.40.0` | Requires | no | 1 | python3.12-opentelemetry_sdk |
| `puppet-agent` | Requires | no | 1 | puppet-foreman_scap_client |
| `drpm-devel` | BuildRequires | no | 1 | createrepo_c |
| `python3.11-drf-spectacular` | Requires | no | 1 | python3.11-galaxy-ng |
| `python3.12-poetry_core == 2.4.0` | Requires | no | 1 | python3.12-poetry |
| `python3.12-pypi-attestations == 0.0.28` | Requires | no | 1 | python3.12-pulp-python |
| `python3.11-six` | Requires | no | 1 | python3.11-pyjwkest |
| `rh-python38` | Requires | no | 1 | %scl_name |
| `pkgconfig(zck)` | BuildRequires | yes | 1 | createrepo_c |
| `python3.11-django-auth-ldap` | Requires | no | 1 | python3.11-galaxy-ng |
| `python3.12-pydantic-core == 2.46.4` | Requires | no | 1 | python3.12-pydantic |

## Version Mismatches

Dependencies found in EL10 but at a version that doesn't satisfy constraints.

| Dependency | Available | Source | Required By (count) | Constraints |
|-----------|-----------|--------|--------------------|-----------|
| `python3.12-cryptography` | 49.0.0 | BaseOS | 3 | < 47.0; < 47; < 49 |
| `ruby` | 3.3.10 | AppStream | 2 | < 3.1; < 3.1 |
| `python3.12-createrepo_c` | 1.1.2 | AppStream | 1 | >= 1.2.1 |
| `libcomps` | 0.1.21 | BaseOS | 2 | = 0.1.23-1.el10; = 0.1.23-1.el10 |
| `rubygem-pg` | 1.5.4 | AppStream | 1 | = 1.6.3-2.el10 |
| `python3.12-libcomps` | 0.1.21 | BaseOS | 1 | >= 0.1.23 |
| `python3.12-requests` | 2.32.4 | BaseOS | 1 | >= 2.33.0 |
| `python3.12-setuptools-rust` | 1.10.2 | CRB | 1 | >= 1.11.0 |
| `ansible-core` | 2.16.19 | AppStream | 1 | = 1:2.16.14-3.el10 |
| `python3.12-typing-extensions` | 4.9.0 | BaseOS | 5 | >= 4.14.1; >= 4.14.1; >= 4.14.1 |
| `python3.12-attrs` | 23.2.0 | BaseOS | 1 | < 23 |
| `python3.12-protobuf` | 3.19.6 | AppStream | 3 | >= 5.29.6; >= 5; >= 4.21.1 |
| `python3.12-installer` | 0.7.0 | CRB | 1 | >= 1.0.0 |
| `python3.12-urllib3` | 1.26.19 | BaseOS | 2 | >= 2.2.2; >= 2 |
| `python3.12-pathspec` | 0.12.1 | CRB | 1 | >= 1.0.0 |
| `python3.12-cffi` | 1.16.0 | BaseOS | 2 | >= 2.0.0; >= 2.0.0 |
| `createrepo_c-libs` | 1.1.2 | AppStream | 3 | = 1.2.1-2.el10; = 1.2.1-2.el10; = 1.2.1-2.el10 |

## Unresolved Macros

Dependencies containing unexpanded RPM macros.

| Raw | Spec | Package |
|-----|------|---------|
| `python%{python3_pkgversion}-devel` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/client/python-psutil/python-psutil.spec` | python-psutil |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-ansi/rubygem-ansi.spec` | rubygem-ansi-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-bootstrap-sass/rubygem-bootstrap-sass.spec` | rubygem-bootstrap-sass-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-bundler_ext/rubygem-bundler_ext.spec` | rubygem-bundler_ext-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-concurrent-ruby-edge/rubygem-concurrent-ruby-edge.spec` | rubygem-concurrent-ruby-edge-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-deacon/rubygem-deacon.spec` | rubygem-deacon-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-httpclient/rubygem-httpclient.spec` | rubygem-httpclient-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-jquery-ui-rails/rubygem-jquery-ui-rails.spec` | rubygem-jquery-ui-rails-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-kafo_parsers/rubygem-kafo_parsers.spec` | rubygem-kafo_parsers-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-little-plugger/rubygem-little-plugger.spec` | rubygem-little-plugger-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-net_http_unix/rubygem-net_http_unix.spec` | rubygem-net_http_unix-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-netrc/rubygem-netrc.spec` | rubygem-netrc-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-powerbar/rubygem-powerbar.spec` | rubygem-powerbar-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-promise.rb/rubygem-promise.rb.spec` | rubygem-promise.rb-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-rack-jsonp/rubygem-rack-jsonp.spec` | rubygem-rack-jsonp-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-redis/rubygem-redis.spec` | rubygem-redis-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-retriable/rubygem-retriable.spec` | rubygem-retriable-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-rsec/rubygem-rsec.spec` | rubygem-rsec-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-sass-rails/rubygem-sass-rails.spec` | rubygem-sass-rails-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-sassc-rails/rubygem-sassc-rails.spec` | rubygem-sassc-rails-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-uber/rubygem-uber.spec` | rubygem-uber-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/foreman/rubygem-websocket-extensions/rubygem-websocket-extensions.spec` | rubygem-websocket-extensions-doc |
| `python%{python3_version}dist(pexpect)` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/ansible-runner/ansible-runner.spec` | python3.12-ansible-runner |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-aws-eventstream/rubygem-aws-eventstream.spec` | rubygem-aws-eventstream-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-aws-sigv4/rubygem-aws-sigv4.spec` | rubygem-aws-sigv4-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-azure_mgmt_compute/rubygem-azure_mgmt_compute.spec` | rubygem-azure_mgmt_compute-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-azure_mgmt_network/rubygem-azure_mgmt_network.spec` | rubygem-azure_mgmt_network-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-azure_mgmt_resources/rubygem-azure_mgmt_resources.spec` | rubygem-azure_mgmt_resources-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-azure_mgmt_storage/rubygem-azure_mgmt_storage.spec` | rubygem-azure_mgmt_storage-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-azure_mgmt_subscriptions/rubygem-azure_mgmt_subscriptions.spec` | rubygem-azure_mgmt_subscriptions-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-bcrypt_pbkdf/rubygem-bcrypt_pbkdf.spec` | rubygem-bcrypt_pbkdf-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-chunky_png/rubygem-chunky_png.spec` | rubygem-chunky_png-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-down/rubygem-down.spec` | rubygem-down-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-dry-equalizer/rubygem-dry-equalizer.spec` | rubygem-dry-equalizer-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-dry-inflector/rubygem-dry-inflector.spec` | rubygem-dry-inflector-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-dry-initializer/rubygem-dry-initializer.spec` | rubygem-dry-initializer-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-dry-logic/rubygem-dry-logic.spec` | rubygem-dry-logic-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-dry-types/rubygem-dry-types.spec` | rubygem-dry-types-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-ed25519/rubygem-ed25519.spec` | rubygem-ed25519-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-foreman_datacenter/rubygem-foreman_datacenter.spec` | rubygem-foreman_datacenter-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-foreman_git_templates/rubygem-foreman_git_templates.spec` | rubygem-foreman_git_templates-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-foreman_graphite/rubygem-foreman_graphite.spec` | rubygem-foreman_graphite-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-foreman_host_extra_validator/rubygem-foreman_host_extra_validator.spec` | rubygem-foreman_host_extra_validator-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-foreman_omaha/rubygem-foreman_omaha.spec` | rubygem-foreman_omaha-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-foreman_vmwareannotations/rubygem-foreman_vmwareannotations.spec` | rubygem-foreman_vmwareannotations-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-graphite-api/rubygem-graphite-api.spec` | rubygem-graphite-api-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-http_parser.rb/rubygem-http_parser.rb.spec` | rubygem-http_parser.rb-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-httparty/rubygem-httparty.spec` | rubygem-httparty-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-infoblox/rubygem-infoblox.spec` | rubygem-infoblox-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-interactor/rubygem-interactor.spec` | rubygem-interactor-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-jquery-matchheight-rails/rubygem-jquery-matchheight-rails.spec` | rubygem-jquery-matchheight-rails-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-logify/rubygem-logify.spec` | rubygem-logify-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-mqtt/rubygem-mqtt.spec` | rubygem-mqtt-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-ms_rest/rubygem-ms_rest.spec` | rubygem-ms_rest-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-ms_rest_azure/rubygem-ms_rest_azure.spec` | rubygem-ms_rest_azure-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-multi_xml/rubygem-multi_xml.spec` | rubygem-multi_xml-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-net-ssh-gateway/rubygem-net-ssh-gateway.spec` | rubygem-net-ssh-gateway-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-openscap/rubygem-openscap.spec` | rubygem-openscap-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-openscap_parser/rubygem-openscap_parser.spec` | rubygem-openscap_parser-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-parse-cron/rubygem-parse-cron.spec` | rubygem-parse-cron-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-polyglot/rubygem-polyglot.spec` | rubygem-polyglot-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-rchardet/rubygem-rchardet.spec` | rubygem-rchardet-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-route53/rubygem-route53.spec` | rubygem-route53-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-rprogram/rubygem-rprogram.spec` | rubygem-rprogram-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-rqrcode/rubygem-rqrcode.spec` | rubygem-rqrcode-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-ruby-hmac/rubygem-ruby-hmac.spec` | rubygem-ruby-hmac-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-ruby-nmap/rubygem-ruby-nmap.spec` | rubygem-ruby-nmap-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-timeliness/rubygem-timeliness.spec` | rubygem-timeliness-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-vault/rubygem-vault.spec` | rubygem-vault-doc |
| `%{pkg_name}` | `/home/zhunting/upstream/el10_rebuild/workspace/foreman-packaging/packages/plugins/rubygem-zscheduler/rubygem-zscheduler.spec` | rubygem-zscheduler-doc |
| `%{scl}-python3.12-devel` | `/home/zhunting/upstream/el10_rebuild/workspace/pulpcore-packaging/packages/libcomps/libcomps.spec` | %{scl}-python3.12-libcomps |
| `%{scl}-python3.12-setuptools` | `/home/zhunting/upstream/el10_rebuild/workspace/pulpcore-packaging/packages/libcomps/libcomps.spec` | %{scl}-python3.12-libcomps |
| `%{scl}-python3.12-pip` | `/home/zhunting/upstream/el10_rebuild/workspace/pulpcore-packaging/packages/libcomps/libcomps.spec` | %{scl}-python3.12-libcomps |
| `%{scl_runtime}` | `/home/zhunting/upstream/el10_rebuild/workspace/pulpcore-packaging/packages/tfm-pulpcore/tfm-pulpcore.spec` | %scl_name-build |

