use std::time::Duration;

/// Subscription descriptions — interpreted in `runtime.rs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sub<M> {
    None,
    Tick { id: u64, every: Duration },
    Stream { id: u64, name: &'static str },
    WebSocket { id: u64, url: &'static str },
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

    pub fn tick(id: u64, every: Duration) -> Self {
        Self::Tick { id, every }
    }

    pub fn stream(id: u64, name: &'static str) -> Self {
        Self::Stream { id, name }
    }

    pub fn websocket(id: u64, url: &'static str) -> Self {
        Self::WebSocket { id, url }
    }

    pub fn map<N>(self, map: fn(M) -> N) -> Sub<N> {
        match self {
            Self::None => Sub::None,
            Self::Tick { id, every } => Sub::Tick { id, every },
            Self::Stream { id, name } => Sub::Stream { id, name },
            Self::WebSocket { id, url } => Sub::WebSocket { id, url },
            Self::Batch(items) => Sub::Batch(items.into_iter().map(|s| s.map(map)).collect()),
            Self::MapMsg { .. } => Sub::None,
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

    #[test]
    fn tick_has_stable_id() {
        let sub = Sub::<u32>::tick(42, Duration::from_secs(1));
        assert_eq!(sub.id(), Some(42));
    }
}
