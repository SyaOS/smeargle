use handlebars::Handlebars;
use surf::Client;

#[derive(Clone)]
pub(crate) struct State {
    pub handlebars: Handlebars<'static>,
    pub client: Client,
}
