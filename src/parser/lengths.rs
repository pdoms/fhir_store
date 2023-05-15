

/// the byte offset in the memory block of the parser.
pub type LengthLocation = usize;
/// The actual length, which can be accumulated during parsing.
pub type LengthLength = u16;
///A Type that represents an Length representation of a fhir object. Mainly
///used for parsing.
pub type Length = (LengthLocation, LengthLength);

///This struct keeps lengths of object. It is stacked based to handle nesting.
#[derive(Default)]
pub struct Lengths(Vec<Length>);

impl Lengths {
    ///returns the number of Length Items in the [`Vec`]
    pub fn len(&self) -> usize {
        return self.0.len()
    }

    ///gets the last [`Length`]. Returns [`None`] if there are no items.
    pub fn last(&mut self) -> Option<Length> {
        if self.len() > 0 {
            match self.0.last()  {
                Some(l) => Some(*l),
                None => None
            }
        } else {
            None
        }
    }

    /// adds to the last [`Length`] if it exists and returns true, otherwise it returns false.
    pub fn add_to_last(&mut self, val: u16) -> bool {
        if self.len() > 0 {
            // we know that there is a last item
            let last = self.0.last_mut().unwrap();
            last.1 += val;
            true
        } else {
            false
        }
    }

    ///if [`Lengths`] is empty, it pushes a [`LengthLocation`] and 0 as a [`Length`] into the
    ///underlying [`Vec`]. Otherwise, instead of setting the initial length to 0, it is set to the
    ///length of the previous item.
    pub fn push(&mut self, location: usize) {
        if self.len() == 0 {
            self.0.push((location, 0));
        } else {
            // we know that there is a last item
            let last = self.last().unwrap();
            self.0.push((location, last.1))
        }
    }

    ///Pops the last element from the stack and returns it in an [`Option`]. In additon, it adds the popped length
    ///to the element that is now on top of the stack. If the stack is empty [`None`] is returned.
    pub fn pop(&mut self) -> Option<Length> {
        if self.len() > 0 {
            // we know that there is a last item
            let val = self.0.pop().unwrap();
            self.add_to_last(val.1);
            Some(val)
        } else {
            None
        }
    }
}