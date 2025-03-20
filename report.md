# Структура проекта и руководство по добавлению новых WS-сообщений

## 📁 Структура проекта
```bash
.
├── Cargo.toml          # Зависимости Rust
├── config/             # Конфигурация приложения
│   └── config.toml
├── src/
│   ├── main.rs         # Точка входа, роутинг и инициализация сервера
│   ├── handlers.rs     # Обработчики WebSocket-соединений
│   ├── models.rs       # Структуры данных и AppState
│   ├── commands/       # Паттерн Command для обработки действий
│   ├── devices/        # Логика устройств (реализация Device)
│   ├── events/         # EventBus (не используется, требует доработки)
│   ├── metrics/        # Экспорт метрик в Prometheus
│   └── utils.rs        # Вспомогательные функции
└── static/             # Веб-интерфейс
    ├── index.html
    ├── styles.css
    └── app.js
```

## 🛠️ Как добавить новые WS-сообщения

### 1. Расширьте `WsMessage` (models.rs)
Добавьте новый вариант в enum:
```rust
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    // ... существующие варианты
    NewCommand { 
        device_id: String,
        param: u32 
    },
}
```

### 2. Реализуйте обработку в handlers.rs
Для устройств:
```rust
async fn handle_device(...) {
    // В цикле обработки сообщений
    if let Ok(WsMessage::NewCommand { device_id, param }) = serde_json::from_str(text) {
        // Логика обработки
    }
}
```

Для клиентов:
```rust
async fn handle_client(...) {
    match command {
        // ... существующие кейсы
        WsMessage::NewCommand { .. } => {
            // Рассылка команды устройству
        }
    }
}
```

### 3. Добавьте CommandHandler (commands/mod.rs)
Создайте новую структуру и реализуйте трейт:
```rust
pub struct NewCommandHandler {
    param: u32
}

#[async_trait]
impl CommandHandler for NewCommandHandler {
    async fn execute(&self, state: Arc<AppState>, device_id: &str) -> Result<(), AppError> {
        // Изменение состояния устройства
    }
}

// Регистрация фабрики через inventory::submit!
```

### 4. Обновите валидацию (опционально)
В методе `validate()` модели `WsMessage` добавьте проверки для новых полей.

### 5. Интеграция с фронтендом (app.js)
Добавьте обработку в классе `LampController`:
```js
sendNewCommand(param) {
    this.sendCommand({
        type: 'new_command',
        device_id: 'ESP32_LAMP_001',
        param: param
    });
}
```

## ⚠️ Важные моменты
1. **Очистка кода**: Удалите неиспользуемые элементы (EventBus, create_set_power_command)
2. **Метрики**: Добавьте отслеживание новых команд в metrics.rs
3. **Безопасность**: Реализуйте валидацию для новых параметров
4. **Тестирование**: Протестируйте через Swagger UI (/docs) и веб-интерфейс

Пример потока данных для новой команды:
```
Frontend (JS) → WebSocket → handle_client → CommandHandler → Device → StatusUpdate → Broadcast → Frontend
```