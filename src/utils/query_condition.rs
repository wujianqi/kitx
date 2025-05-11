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
    /// Creates a new query condition wrapper
    /// 
    /// # Arguments
    /// * `query_fn` - Closure defining query conditions
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

pub struct Shared<Q>(Arc<Q>);

impl<Q> Shared<Q> {
    pub fn new(query: Q) -> Self {
        Self(Arc::new(query))
    }

    pub fn share(&self) -> Arc<Q> {
        Arc::clone(&self.0)
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn inner(self) -> Arc<Q> {
        self.0
    }
}