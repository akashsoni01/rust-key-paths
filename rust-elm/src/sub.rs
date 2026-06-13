use std::time::Duration;

/// Subscription descriptions — interpreted in [`crate::subscription`] / [`crate::runtime`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sub<M> {
    None,
    Tick {
        id: u64,
        every: Duration,
        produce: fn() -> M,
    },
    Stream {
        id: u64,
        name: &'static str,
        every: Duration,
        produce: fn() -> M,
    },
    WebSocket {
        id: u64,
        url: &'static str,
        every: Duration,
        produce: fn() -> M,
    },
    MapMsg {
        inner: Box<Sub<()>>,
        map: fn(()) -> M,
    },
    Batch(Vec<Sub<M>>),
}

impl<M> Sub<M> {
    pub fn none() -> Self {
        Self::None
    }

    pub fn tick(id: u64, every: Duration, produce: fn() -> M) -> Self {
        Self::Tick { id, every, produce }
    }

    pub fn stream(id: u64, name: &'static str, every: Duration, produce: fn() -> M) -> Self {
        Self::Stream {
            id,
            name,
            every,
            produce,
        }
    }

    pub fn websocket(id: u64, url: &'static str, every: Duration, produce: fn() -> M) -> Self {
        Self::WebSocket {
            id,
            url,
            every,
            produce,
        }
    }

    /// Wrap a `Sub<()>` and map each firing into `M` (Elm `Sub.map`).
    pub fn map_msg(inner: Sub<()>, map: fn(()) -> M) -> Self {
        Self::MapMsg {
            inner: Box::new(inner),
            map,
        }
    }

    /// Map batched subscription outputs. For leaf ticks/streams, use explicit `produce` fns or
    /// [`Self::map_msg`] at the target action type.
    pub fn map<N>(self, _f: fn(M) -> N) -> Sub<N> {
        match self {
            Self::None => Sub::None,
            Self::Batch(items) => Sub::Batch(items.into_iter().map(|s| s.map(_f)).collect()),
            Self::MapMsg { .. } | Self::Tick { .. } | Self::Stream { .. } | Self::WebSocket { .. } => {
                Sub::None
            }
        }
    }

    pub fn batch(subs: impl IntoIterator<Item = Sub<M>>) -> Self {
        let subs: Vec<_> = subs.into_iter().collect();
        if subs.is_empty() {
            Self::None
        } else if subs.len() == 1 {
            subs.into_iter().next().unwrap()
        } else {
            Self::Batch(subs)
        }
    }

    pub fn id(&self) -> Option<u64> {
        match self {
            Self::None => None,
            Self::Tick { id, .. } | Self::Stream { id, .. } | Self::WebSocket { id, .. } => {
                Some(*id)
            }
            Self::MapMsg { inner, .. } => inner.id(),
            Self::Batch(items) => items.first().and_then(Sub::id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero() -> i32 {
        0
    }

    fn unit() {}

    fn ping(_: ()) -> i32 {
        7
    }

    #[test]
    fn tick_has_stable_id() {
        let sub = Sub::<i32>::tick(42, Duration::from_secs(1), zero);
        assert_eq!(sub.id(), Some(42));
    }

    #[test]
    fn map_msg_wraps_unit_tick() {
        let sub = Sub::map_msg(Sub::tick(1, Duration::from_millis(1), unit), ping);
        assert_eq!(sub.id(), Some(1));
        let Sub::MapMsg { map, .. } = sub else {
            panic!("expected map_msg");
        };
        assert_eq!(map(()), 7);
    }
}
