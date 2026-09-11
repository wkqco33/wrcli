# wrcli Config — Viper 패리티 로드맵

`wrcli::Config`를 Go [viper](https://github.com/spf13/viper) 수준으로 확장하기 위한
계획과 설계 결정. 각 단계는 AGENTS.md의 TDD(Red → Green → Refactor)를 따르며,
기능 변경 시 `docs/GUIDE.md`·`README.md`·`AGENTS.md`를 함께 갱신한다.

## 티어

| 티어 | 범위 | 상태 |
| --- | --- | --- |
| T1 저위험 | Set/IsSet/Alias/구분자·env replacer/추가 getter | 완료 |
| T2 중위험 | AllKeys/AllSettings/Map getter/Sub/Merge/Write/신규 포맷 | 완료 |
| T3 고위험 | Unmarshal(serde), WatchConfig | 예정 |
| 비목표 | Remote(Etcd/Consul/Firestore/NATS), crypt | 제외 |

## 설계 결정 (승인됨)

- **D1 Map 표현**: 평탄 저장을 유지하고 `all_settings`/`get_string_map*`에서 점 키를
  중첩으로 재구성한다. `ConfigValue`에 Map 변형을 추가하지 않는다.
- **D2 Unmarshal**: 옵션 `serde` 피처 + `ConfigValue` 기반 `Deserializer`(포맷 비의존).
- **D3 Watch**: `watch_config(&mut self) -> ConfigWatcher` 핸들 + 콜백에 스냅샷 `Config`
  전달. 1차는 의존성 없는 mtime 폴링.
- **D4 empty env**: `allow_empty_env(bool)` 추가, 기본은 현재 동작(empty=값) 유지.
  Viper 기본(empty=unset)은 옵트인. 1.0에서 전환 검토.
- **D5 대소문자**: `case_insensitive_keys(bool)` 옵트인(기본 off).
- **D6 YAML 백엔드**: `noyalib = "0.0"` 성숙도 검토 후 교체.

## 우선순위 레이어 (Set 추가)

```text
explicit(set) → flag → env → file → default
```

## 단계

### Phase 1 — T1 (완료)

- `set`, `is_set`, `register_alias`, `set_key_delimiter`, `set_env_key_replacer`,
  `allow_empty_env`.
- getter: `get_int64`, `get_uint`, `get_duration`, `get_time`, `get_size_in_bytes`,
  `get_string_slice`.
- 테스트: `tests/config.rs` Phase 1 섹션 17개.

### Phase 2 — T2 데이터 접근 (완료)

- `all_keys`, `all_settings`, `get_string_map`/`get_string_map_string`/
  `get_string_map_string_slice`, `sub`.
- `read_config(reader)`, `merge_in_config(path)`, `merge_config_map`.
- 신규 타입 `SettingsMap`/`SettingsEntry`, 에러 `ConfigTypeNotSet`.
- 테스트: `tests/config.rs` Phase 2 섹션 8개 + `settings.rs` 단위 2개.

### Phase 3 — T2 쓰기 + 포맷 (완료)

- `write_config_as`, `safe_write_config_as` (에러 `ConfigFileExists`).
- 신규 피처/포맷: `ini-config`, `dotenv-config`, `properties-config` (읽기+쓰기).
- 쓰기 지원: TOML, JSON, INI, dotenv, properties.
- 테스트: `tests/config.rs` Phase 3 섹션 6개.

### Phase 4 — T3 Unmarshal (다음)

- `serde` 피처, `Config::unmarshal<T>()`, `unmarshal_key<T>(key)`.

### Phase 5 — T3 Watch

- `watch_config()` + `on_config_change()`, 이후 `notify` 옵션 피처.

### Phase 6 — 선택

- `set_config_permissions`, YAML 쓰기, `case_insensitive_keys`, YAML 백엔드 교체, `debug()`.

## 리스크

| 리스크 | 완화 |
| --- | --- |
| `ConfigValue` 변형 추가 = breaking | D1에 따라 변형 추가 안 함 |
| serde 의존 | 옵션 피처 + 포맷 비의존 Deserializer |
| Watch 동시성 | 스냅샷 클론 콜백, 최후순위 |
| empty env 기본값 차이 | 옵트인 + 문서화 |
| `noyalib 0.0` | Phase 6 교체 검토 |
