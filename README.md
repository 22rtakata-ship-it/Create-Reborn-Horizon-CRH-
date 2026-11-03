# Create for Horizon — экспериментальный порт

**Unofficial experimental port**, maintained by [Delete228](https://github.com/Delete228).
Based on [Create by the Creators of Create](https://github.com/Creators-of-Create/Create).
This is an early creative-mode prototype, not an official Create or Horizon release.
Only the code covered by MIT is adapted; original Create assets are excluded from
the public source and default packages. See [third-party notices](THIRD_PARTY_NOTICES.md).

**Запуск через официальный лаунчер:** [установка, сборка и визуальная проверка](LAUNCHER.md).
**Оригинальная графика:** [положите свой JAR рядом с модом — импорт при запуске](ASSETS.md).
**План и остановки для проверки:** [PLAN.md](PLAN.md).
**Исходные ID и границы формата данных:** [COMPATIBILITY.md](COMPATIBILITY.md).

Отдельный начальный порт Create на Rust. Папки `../Create` и `../horizon` используются
как исходные материалы и не изменяются. Версии закреплены в `compatibility.json`.

Реализованы базовые соединения валов и шестерён, скорости и нагрузка сети, мотор,
цикл механического пресса и обработка предметов на депо. Это ранний прототип,
пока для творческого режима. Полного Create, совместимости сохранений Java-мода
и совместимости его аддонов здесь ещё нет.

## Что находится в проекте

- `mixins/create/engine/` — Rust-ядро кинетики и цикла пресса без зависимостей от игры.
- `mixins/create/assets/` — локальный импортёр ресурсов из предоставленного пользователем JAR.
- `mixins/create/runtime.rs` — адаптация к настоящим блокам, NBT, депо, рецептам и рендерингу Horizon.
- `mixins/create/horizon-mixin.toml` — десять именованных hooks, четыре закреплённых mixin
  для ресурсов и клиентского результата клика; отдельные crates движка и импортёра.
- `content/` — шесть блоков, три пластины, три рецепта и вкладка творческого инвентаря.
  Начальные модели написаны отдельно и используют ссылки на vanilla-текстуры.
- `tools/` — установка локальных инструментов, импорт, сборка, проверки и запуск.
- `.local/port-work/` — результат настоящего `horizon-port-pack`; оригинальные jar,
  ресурсы, игровые миры и временные сборки остаются локальными и исключены из Git.
- `dist/` — проверенные архивы кода и небольшого контентного набора.
- `horizon-native.json` — исходный нативный пакет для установки по GitHub-ссылке;
  `mixins/create/data/content.json` содержит весь публичный контент этого пакета.

## Подготовка и сборка

Обычная установка не требует соседних репозиториев Create/Horizon, нашего
bootstrap или ручных поправок. Создайте instance в Horizon Launcher, установите
Create через **Mods → Add from GitHub**, затем соберите нативные моды.
Через консоль с уже установленным лаунчером можно выполнить:

```sh
python3 tools/launcher.py --instance "Create Native" --github https://github.com/Delete228/create-horizon
```

Для повторной сборки установленного пакета достаточно команды без `--github`.
Для разработки уже включённого Create добавьте `--source`: соберётся текущая
рабочая копия. Оба режима вызывают штатный `horizon-launcher native-build`;
компилятор, исходники Horizon, Mixins и кэши обслуживает сам лаунчер. Подробные
шаги, закреплённая проверенная версия и ограничения находятся в [LAUNCHER.md](LAUNCHER.md).

### Низкоуровневые инструменты разработки

Команды выполняются из этой папки; нужны Python 3.9+, Git и системный C/C++ linker.
Для Java-оракула нужен JDK. На этой машине установлен JDK 25.

Для новой рабочей копии сначала подготовьте соседние репозитории на закреплённых
коммитах (уже существующие рабочие копии с изменениями не переключайте):

```sh
git clone https://github.com/Delete228/create-horizon.git
cd create-horizon
git clone https://github.com/horizon-rust/horizon.git ../horizon
git -C ../horizon checkout fc02103818c212920fd24aa34a2b2f91ba49e56f
git clone https://github.com/Creators-of-Create/Create.git ../Create
git -C ../Create checkout ac0c444d9828da3453ae8cc65338e8de063286fb
```

```sh
python3 tools/bootstrap.py
python3 tools/cargo.py build --manifest-path ../horizon/Cargo.toml --locked -p horizon-port-pack -p cargo-horizon-mod
python3 tools/content.py
python3 tools/stage.py check
python3 tools/stage.py build
python3 tools/package.py
```

Rust 1.99.1 устанавливается в `.tools/rust/`, Cargo использует `.cache/cargo/`.
Глобальные Rust-настройки и профиль оболочки не меняются. `stage.py` экспортирует
закреплённый Git-коммит Horizon в `.local/horizon-source/` и использует официальный
`horizon-mixins plan/apply`. Собирается `.local/stage/`; результат —
`target/fast/horizon-client`. Звук и контроллеры в этой разработческой сборке отключены.

На macOS повторный `apply` требует новый каталог. Как лаунчер Horizon, скрипт
применяет mixins во временный каталог и переносит только изменившиеся файлы в
постоянную сборочную папку. Документационная ссылка `CLAUDE.md` при экспорте опускается.

**Версия инструмента:** пакет закрепляет опубликованный коммит Mixins
`a1e13326306c6bc074b11e6a4926deb551c4083e` с исправлением `recursion_limit`
из [PR #26](https://github.com/horizon-rust/horizon-mixins/pull/26). Его получает
неизменённый лаунчер из официального Git-репозитория; `setup_mixins.py` также
собирает этот коммит без правки исходников. Это пока ветка непринятого PR.
Готовность к каталогу описана в [REGISTRY_READINESS.md](REGISTRY_READINESS.md).

## Что сделал horizon-port-pack

Для начального импорта используется опубликованный Create **6.0.10** на NeoForge
1.21.1. Поведение теперь переносится с коммита того же релиза **6.0.10**;
инструменты читают его через Git, не переключая соседнюю рабочую копию. Исходный jar
проверяется по SHA-512 из метаданных закреплённого релиза Modrinth.

Запускаются настоящие шаги `import`, `inventory --offline`, `carry ... create`,
`report` и `check`. Отчёт помечает Create как **partial**. При первом запуске:
8913 перенесённых файлов, 542 блока и 142 предмета по vanilla-шаблонам; Java-логика
не переносится. Этот полный набор не подключается к игровому прототипу: активен
только явно реализованный небольшой набор из `content/`.

Разработческий импорт port-pack запускается отдельно: `tools/import_create.py --jar PATH`
для своего JAR либо `tools/import_create.py --download-bootstrap` для явного скачивания
закреплённого входа 6.0.10. Это не требуется для сборки и запуска прототипа.
Другой JAR можно исследовать через `tools/import_create.py --jar PATH`, однако
игровой импорт поддерживает только закреплённый релиз 6.0.10.

Оригинальные ресурсы Create имеют All Rights Reserved. Обычный пакет их не содержит.
Пользователь сам помещает JAR Create 6.0.10 NeoForge 1.21.1 в
`horizon-mods/create-resources/` своего инстанса; мод импортирует нужные ресурсы
автоматически при запуске и перезагрузке ресурспаков. JAR проверяется по SHA-256,
распакованный набор остаётся в `.horizon-cache/create-assets/`. Подробности и
границы лицензии — в [ASSETS.md](ASSETS.md).
Для отдельного разработческого мира:

```sh
python3 tools/play.py --create-jar /path/to/create-1.21.1-6.0.10.jar
```

Этот запуск не создаёт архивов с чужими ресурсами. Атрибуция MIT-кода и границы
лицензий описаны в `THIRD_PARTY_NOTICES.md`.

## Проверки

```sh
python3 tools/cargo.py test --workspace --target-dir target/core
python3 tools/cargo.py clippy --workspace --all-targets --target-dir target/core -- -D warnings
python3 tools/cargo.py run --example factory --target-dir target/core
python3 tools/oracle.py
python3 tools/placement_oracle.py
```

Оракул компилирует **неизменённый** исходный `KineticNetwork.java` с минимальным
адаптером блока/мира и сравнивает 75 сценариев мощности и нагрузки с Rust.
Второй оракул извлекает тела методов размещения из того же Java-коммита через
AST JDK и сравнивает 27 648 комбинаций для вала, шестерней и мотора.
Это не полная проверка поведения мода в NeoForge. Unit-тесты дополнительно
проверяют передаточные отношения, разрыв сети, перегрузку и цикл пресса.

## Проверка в игре и обычный запуск

Подготовленные ресурсы 26.3 на этой машине найдены в каталоге установленного
Horizon Nightly под `~/Library/Application Support/horizon-launcher/versions/`.
Путь заканчивается на `.local/horizon-client/26.3`.

```sh
python3 tools/smoke.py --resources "/полный/путь/к/.local/horizon-client/26.3"
python3 tools/smoke.py --resources "/полный/путь/к/.local/horizon-client/26.3" --offscreen
python3 tools/play.py --resources "/полный/путь/к/.local/horizon-client/26.3"
```

`smoke.py` создаёт отдельные homes, копию ресурсов и новый тестовый мир. Он
проверяет железные пластины, остановку при разрыве вала и медные пластины после
восстановления соединения, затем сохранение и загрузку незавершённого цикла
золотых пластин. Дополнительно проверяются размещение и прямой обмен с депо
без меню, количество и компоненты предметов через сохранение. В режиме
`--offscreen` сохраняются снимки рендера. Логи и результаты — в `.local/smoke-*/`.
`play.py` использует собственный постоянный мир `.local/play-home/`; путь к
ресурсам после первого запуска сохраняется в локальной конфигурации.

На этой машине сборка, проверки и оба игровых сценария уже выполнены:
см. [результаты первого этапа](VALIDATION.md). Путь к ресурсам сохранён, поэтому
для обычного запуска достаточно `python3 tools/play.py`.

Для оригинальных ресурсов передайте `--create-jar /path/to/create-1.21.1-6.0.10.jar`
при первом запуске `play.py`; при следующих запусках достаточно оставленного JAR.

В творческом инвентаре есть вкладка Create. Мотор по умолчанию выдаёт 16 RPM.
Для первого стенда можно использовать команды:

```text
/setblock 0 66 0 create:creative_motor[facing=east,speed=16]
/setblock 1 66 0 create:shaft[axis=x]
/setblock 2 66 0 create:cogwheel[axis=x]
/setblock 2 66 1 create:cogwheel[axis=x]
/setblock 3 66 1 create:mechanical_press[facing=east]
/setblock 3 64 1 create:depot[facing=up]
/give @s minecraft:iron_ingot 8
```

Валы и шестерни задают ось через `axis`. Мотор и пресс — через `facing`.
Пресс работает с депо на два блока ниже. Положите слитки в его интерфейс.
Депо открывает простой односекционный
интерфейс; предметы могут подавать и забирать воронки. Поддерживаются bulk-рецепты
с одним входом и одним гарантированным выходом: железо, золото и медь.

## Границы первого этапа

- Сеть ограничена 256 загруженными узлами. Кэширование и сохранение сети через
  выгрузку чанков предстоят; пока она пересчитывается при тиках механизмов.
- Есть соосное соединение, малые шестерни в одной плоскости и диагональное
  соединение большой/малой шестерни с отношением 2:1.
- Конфликтующие источники и превышение скорости останавливают сеть; оригинальное
  разрушение конфликтующих блоков пока не воспроизводится.
- Настройка мотора пока через `speed` в состоянии блока, без интерфейса Create.
- Модели, формы и анимации предварительные. Точное визуальное соответствие ещё не заявлено.
- Обработка на лентах, в мире и в чаше, случайные выходы, прочие механизмы,
  contraptions, поезда, Ponder и аддоны — следующие этапы.
- Полноценный тест нескольких сетевых клиентов ещё требуется.
- В текущем Horizon команда `/item replace block` ещё не работает с mod-контейнерами;
  используйте интерфейс депо, воронки или `horizon-ctl container` в тестовом мире.

Официальные руководства: [Mods](https://github.com/horizon-rust/horizon/blob/main/docs/mods.md),
[Native mods](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md),
[Porting packs](https://github.com/horizon-rust/horizon/blob/main/docs/modpacks.md),
[Native archive format](https://github.com/horizon-rust/horizon/blob/main/docs/api/mod-files.md).
