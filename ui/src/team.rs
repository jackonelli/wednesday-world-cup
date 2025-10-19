use wwc_core::team::Team;

pub(crate) fn format_team_flag(team: &Team) -> String {
    format!("tournament-group__flag flag-icon flag-icon-{}", team.iso2())
}
