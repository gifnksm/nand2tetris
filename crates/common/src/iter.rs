use either::Either;

pub trait IteratorExt: Iterator {
    fn prependable(self) -> Prependable<Self>
    where
        Self: Sized,
    {
        Prependable {
            iter: self,
            next: None,
        }
    }
}

impl<I> IteratorExt for I where I: Iterator {}

#[derive(Debug)]
pub struct Prependable<I>
where
    I: Iterator,
{
    iter: I,
    next: Option<I::Item>,
}

impl<I> Iterator for Prependable<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().or_else(|| self.iter.next())
    }
}

impl<I> Prependable<I>
where
    I: Iterator,
{
    pub fn prepend(&mut self, item: I::Item)
    where
        I: Iterator,
    {
        assert!(self.next.is_none());
        self.next = Some(item);
    }

    pub fn peek(&mut self) -> Option<&I::Item>
    where
        I: Iterator,
    {
        if self.next.is_none() {
            self.next = self.iter.next();
        }
        self.next.as_ref()
    }
}

pub trait TryIterator<T, E1>: Iterator<Item = Result<T, E1>> {
    fn map_ok<U, F>(self, f: F) -> MapOk<Self, F>
    where
        Self: Sized,
        F: FnMut(T) -> U,
    {
        MapOk { iter: self, f }
    }

    fn try_map_ok<E2, F>(self, f: F) -> TryMapOk<Self, F>
    where
        Self: Sized,
        F: FnMut(T) -> Result<T, E2>,
    {
        TryMapOk { iter: self, f }
    }

    fn try_inspect_ok<E2, F>(self, f: F) -> TryInspectOk<Self, F>
    where
        Self: Sized,
        F: FnMut(&T) -> Result<(), E2>,
    {
        TryInspectOk { iter: self, f }
    }

    fn consume_ok(mut self) -> Result<(), E1>
    where
        Self: Sized,
    {
        if let Some(err) = self.find_map(Result::err) {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl<T, E, I> TryIterator<T, E> for I where I: Iterator<Item = Result<T, E>> {}

#[derive(Debug)]
pub struct MapOk<I, F> {
    iter: I,
    f: F,
}

impl<I, F, T, U, E> Iterator for MapOk<I, F>
where
    I: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U,
{
    type Item = Result<U, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|res| res.map(&mut self.f))
    }
}

#[derive(Debug)]
pub struct TryMapOk<I, F> {
    iter: I,
    f: F,
}

impl<I, F, T, E1, E2> Iterator for TryMapOk<I, F>
where
    I: Iterator<Item = Result<T, E1>>,
    F: FnMut(T) -> Result<T, E2>,
{
    type Item = Result<T, Either<E1, E2>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|res| match res {
            Ok(t) => (self.f)(t).map_err(Either::Right),
            Err(e) => Err(Either::Left(e)),
        })
    }
}

#[derive(Debug)]
pub struct TryInspectOk<I, F> {
    iter: I,
    f: F,
}

impl<I, F, T, E1, E2> Iterator for TryInspectOk<I, F>
where
    I: Iterator<Item = Result<T, E1>>,
    F: FnMut(&T) -> Result<(), E2>,
{
    type Item = Result<T, Either<E1, E2>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|res| match res {
            Ok(t) => (self.f)(&t).map(|_| t).map_err(Either::Right),
            Err(e) => Err(Either::Left(e)),
        })
    }
}
