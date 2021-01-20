use crate::app::Msg;
use seed::{prelude::*, *};
use wwc_core::team::Team;

pub(crate) fn format_team_flag(team: &Team) -> Node<Msg> {
    span![C![format!(
        "tournament-group__flag flag-icon flag-icon-{}",
        team.iso2
    )]]
}
