use actix_web::web::ServiceConfig;

use super::health;

pub(crate) fn services(config: &mut ServiceConfig) {
    health::services(config);
}
