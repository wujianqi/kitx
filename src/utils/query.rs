use std::marker::PhantomData;

/// Creates a dynamic query function.
pub fn dyn_query<'a, F, Q>(query_fn: F) -> Option<Box<dyn Fn(&mut Q) + Send + 'a>>
where
    F: Fn(&mut Q) + Send + 'a
{
    Some(Box::new(query_fn))
}

/// Creates an empty query function.
pub fn empty_query<'a, Q>() -> Option<Box<dyn Fn(&mut Q) + Send + 'a>> {
    None
}

/// A query condition. This is a wrapper around a function that takes a query and returns a boolean.
pub struct QueryCondition<'a, Q, F>
where
    F: Fn(&mut Q) + Send + 'a,
{
    condition: Option<F>,
    _marker: PhantomData<&'a Q>
}

impl<'a, Q, F> QueryCondition<'a, Q, F>
where
    F: Fn(&mut Q) + Send + 'a,
{
    pub fn new(query_fn: Option<F>) -> QueryCondition<'a, Q, F> {
        QueryCondition {
            condition: query_fn,
            _marker: PhantomData
        }
    }

    pub fn take(&mut self) -> Option<F> {
        self.condition.take()
    }
}