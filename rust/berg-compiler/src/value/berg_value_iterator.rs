use value::*;

pub struct BergValueIterator<'a>(Option<BergResult<'a>>);

impl<'a> Iterator for BergValueIterator<'a> {
    type Item = BergResult<'a>;
    fn next(&mut self) -> Option<BergResult<'a>> {
        match self.0 {
            Some(Ok(BergVal::List)) => 
            Some(Ok(BergVal::BlockRef(_))) => unreachable(),
        }
        current
    }
}