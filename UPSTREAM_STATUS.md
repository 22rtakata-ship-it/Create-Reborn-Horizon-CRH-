# Изменения Horizon, нужные порту

Проверено 5 ноября 2026. Порт использует закреплённый опубликованный форк
Horizon. Его native-build автоматически собирает Launcher; пользователю не
нужно править исходники движка вручную. Принятие одного PR не означает, что весь
адаптер уже совместим с последней Nightly: остальные границы нужно переносить
и проверять отдельно.

| Изменение | Для чего нужно | Фактическое состояние |
|---|---|---|
| [Horizon #8872](https://github.com/horizon-rust/horizon/pull/8872) | Именованный урон мобам и частицы с собственным состоянием | Принят и объединён в main |
| [Horizon #8918](https://github.com/horizon-rust/horizon/pull/8918) | Текст и atlas quads в мире с depth test, независимый lifecycle | Обновлённая ветка отправлена; открыт, готов к ревью |
| [Horizon #8924](https://github.com/horizon-rust/horizon/pull/8924) | Сохранение gain/pitch/position звуков, управляемых native-модом | Обновлённая ветка отправлена; открыт, готов к ревью |
| [Horizon #9519](https://github.com/horizon-rust/horizon/pull/9519) | Damage types активного мира, теги, exhaustion и синхронизация | Исправленный follow-up открыт, готов к ревью; [прежний #9076](https://github.com/horizon-rust/horizon/pull/9076) закрыт |
| [Mixins #25](https://github.com/horizon-rust/horizon-mixins/pull/25) | Корректная обработка встроенных crate/tool attributes | Принят; закреплён в кандидате alpha.3 |
| [Mixins #26](https://github.com/horizon-rust/horizon-mixins/pull/26) | Наш вариант того же исправления | Закрыт как дублирующий уже принятое исправление |
| [Horizon #9529](https://github.com/horizon-rust/horizon/pull/9529), [issue #9521](https://github.com/horizon-rust/horizon/issues/9521) | Metadata и post-build данные native-модов рядом с executable внутри macOS `.app` | Исправление отправлено, открыто и готово к ревью; исходный `ensure`, данные и рендер проверены в приватной `.app` |

Это небольшие общие границы движка, отчасти похожие по роли на сервисы mod loader:
их могут использовать другие Rust-моды. Это не перенос всего Forge/NeoForge.
Create-specific рецепты, поведение, интерфейсы и эффекты остаются в моде.
#9519 вообще исправляет загрузку игровых данных, а не добавляет отдельный API
Create. Изменения рассчитаны на сохранение обычных путей Horizon, но абсолютную
безошибочность обещать нельзя: её заменяют review, regression и parity checks.

## Что сделано после замечаний к #9076

Мейнтейнер указал две проблемы: приземление читало стартовые ресурсы вместо
ресурсов мира; полная синхронная загрузка при любом datapack замедляла открытие
мира. Он написал, что исправленную версию можно переоткрыть. Авторский интерфейс
GitHub не показывает Reopen для закрытого PR, поэтому подготовлен связанный
follow-up [#9519](https://github.com/horizon-rust/horizon/pull/9519) из **той же
ветки**, без переписывания истории. В старом обсуждении опубликована
[ссылка на замену](https://github.com/horizon-rust/horizon/pull/9076#issuecomment-6643769160).

Ветка [`data-driven-damage` at 002cf12b](https://github.com/Delete228/horizon/tree/002cf12bf264541bba7306d7f12ee2903249ecc6)
уже отправлена. Приземление использует активный Health catalog; eager full SyncData
нужен только pack с определениями damage types. Остальные packs сохраняют lazy
загрузку, а определения фиксируются на время мира. Добавлена регрессия удаления
всех packs до первого подключения к hub.

На конечном commit прошли 10 профильных resource-тестов и 9 свежих сравнений с
Minecraft 26.3: 69 531 fixture, включая 411 codec cases. Strict all-target Clippy,
форматирование и public-tree/history checks прошли. Полный прежний self-check
сохранён отдельно и не объявляется полностью зелёным: в нём остаются неизменённые
macOS installer fixture и 1-ULP item-drop failure. Проверка control-функции на main
не равна полному прогону main. Английское описание PR сохраняет эти границы.

## GitHub и публикация

Подключённый GitHub распознаёт аккаунт Delete228 и позволяет читать исходники,
обсуждения и CI. Попытки создать/обновить issue или PR в upstream возвращают
`403 Resource not accessible by integration`. SSH push в собственные репозитории
работает. Установка GitHub Desktop для этого не требуется.

Рабочий запасной путь — Firefox с уже выполненным входом. Именно так два драфта
были переведены в ready for review и закрыт дублирующий Mixins PR. После восстановления
управления браузером опубликованы #9519, ссылка в #9076, macOS issue #9521 и PR #9529.
Прежнее сообщение инструмента о блокировке Mac не принимается за доказательство
фактической блокировки экрана пользователем.
Для нового Launcher-исправления [правила Horizon](https://github.com/horizon-rust/horizon/blob/main/AGENTS.md)
требуют: **“Claim before you start”** — сначала issue и `/claim`.
[Claim опубликован](https://github.com/horizon-rust/horizon/issues/9521#issuecomment-6643836877)
до начала production-поправки Launcher.

На момент этой остановки в основном Horizon открыты ровно четыре PR аккаунта
Delete228: #8918, #8924, #9519 и #9529. Все готовы к ревью, черновиков нет.

## Исправление macOS Launcher

В [#9529](https://github.com/horizon-rust/horizon/pull/9529) изменён только
`launcher/src/game_app.rs`, commit `69e6298905d31b615e22696927c957dfb49a736c`.
Metadata обновляется атомарной заменой hardlink независимо от executable;
`native-data` видна через относительную ссылку на папку версии. Отсутствующие
companion-ресурсы убираются, неожиданные реальные данные сохраняются с ошибкой
и штатным fallback на корневой executable.

Прошли восемь профильных тестов в настоящем Launcher, strict all-target Clippy
и полный набор Launcher: 524 passed, 7 прежних ignored. Полный self-check сохраняет
один Python failure в неизменённой installer fixture; узкий контроль повторяет
его на файлах exact main `83ad384f4`. Зелёный прогон всей основной ветки не заявляется.

Отдельный probe с объявленными заглушками только локализации/XML вызвал настоящий
неизменённый `ensure`. Созданный `.app`-процесс прочитал реальные metadata и тестовые
post-build данные через настоящий `native_data::built`. Затем executable из этой
`.app` запустил игру: цифры 128 и белая рамка появились. Кадры до/после включены в PR.
Это не автоматизация кнопки Play установленного Launcher: обновление пользовательской
установки и чистый native-build кандидата остаются отдельной проверкой.

PR #8918, #8924, #9519 и #9529 прошли CI verify и launcher; fork parity требует отдельного
разрешения мейнтейнера и не обозначается пройденным. Ни один открытый PR мы сами
не объединяли в основной Horizon.

Для каталога модов см. [REGISTRY_READINESS.md](REGISTRY_READINESS.md): можно готовить
честно описанный experimental-пакет после полного установочного прогона; называть
текущий порт полной или стабильной реализацией Create преждевременно.
