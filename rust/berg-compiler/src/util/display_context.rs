use std::fmt::*;

// Implement this on the thing you want to be able to format.
pub trait DisplayContext<Context> {
    fn fmt(&self, f: &mut Formatter, context: &Context) -> Result;
    fn disp(&self, context: Context) -> DisplayContextCarrier<Self, Context> {
        DisplayContextCarrier(self, context)
    }
}

pub struct DisplayContextCarrier<'v, Val: DisplayContext<Context> + 'v + ?Sized, Context>(
    pub &'v Val,
    pub Context,
);

impl<'v, Val: DisplayContext<Context> + 'v + ?Sized, Context> Display
    for DisplayContextCarrier<'v, Val, Context> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.0.fmt(f, &self.1)
    }
}
