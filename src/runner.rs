use crate::config::Service;
use crate::spawn_service::spawn_service;
use crate::watcher::run_with_watch;

pub async fn runner(service: Service, color: colored::Color) {
    if service.watch.unwrap_or(false) {
        run_with_watch(service, color).await;
    } else {
        spawn_service(&service, color, true).await;
    }
}
