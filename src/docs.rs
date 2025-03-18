use utoipa::OpenApi;
use crate::models;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Добавьте здесь обработчики
    ),
    components(
        schemas(models::WsMessage, models::DeviceStatus)
    )
)]
pub struct ApiDoc;