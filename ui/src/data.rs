use seed::prelude::*;
use wwc_core::{group::Groups, team::Teams};

pub(crate) async fn get_teams() -> fetch::Result<Teams> {
    Request::new("http://192.168.0.15:8000/get_teams")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub(crate) async fn get_groups() -> fetch::Result<Groups> {
    Request::new("http://192.168.0.15:8000/get_groups")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}
