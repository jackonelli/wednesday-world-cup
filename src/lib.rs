pub mod fair_play;
pub mod game;
pub mod group;
pub mod playoff;
pub mod team;

#[derive(Clone)]
pub struct Date {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
