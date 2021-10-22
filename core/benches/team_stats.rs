// #![feature(test)]
// extern crate test;
// use test::{black_box, Bencher};
// use wwc_core::group::{
//     stats::{GameStat, TableStats},
//     Group,
// };
// #[bench]
// pub fn table_stats(b: &mut Bencher) {
//     let seed = Some(0);
//     let group = Group::random(6, 4, 6, seed);
//     b.iter(|| TableStats::team_stats(black_box(&group)));
// }
