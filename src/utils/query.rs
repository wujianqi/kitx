use std::sync::Arc;
use std::marker::PhantomData;

/// Create an empty query condition.
pub fn empty_query<'a, Q>() -> Box<dyn Fn(&mut Q) + Send + Sync + 'a> {
    Box::new(|_| {})
}

/// A query condition wrapper for concurrent use.
pub struct QueryCondition<'a, Q, F>
where
    F: Fn(&mut Q) + Send + Sync + 'a,
{
    condition: Arc<F>,
    _marker: PhantomData<&'a Q>,
}

impl<'a, Q, F> QueryCondition<'a, Q, F>
where
    F: Fn(&mut Q) + Send + Sync + 'a,
{
    pub fn new(query_fn: F) -> Self {
        QueryCondition {
            condition: Arc::new(query_fn),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> impl Fn(&mut Q) + Send + Sync + 'a {
        let arc_condition = self.condition.clone();
        move |q| arc_condition(q)
    }
}