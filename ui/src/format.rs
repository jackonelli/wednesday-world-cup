use crate::app::Msg;
use seed::prelude::*;

pub(crate) trait Format<'a> {
    type Context;
    fn format(&self, cxt: &Self::Context) -> Node<Msg>;
}
