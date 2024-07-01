/// A function that takes a character and returns a boolean.
pub(crate) struct MatchFunction(pub(crate) Box<dyn Fn(char) -> bool + 'static>);
