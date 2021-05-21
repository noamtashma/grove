use std::marker::PhantomData;
use crate::*;
use super::*;

pub struct Slice<'a, D, T, L> {
    phantom : PhantomData<D>,
    tree : &'a mut T,
    locator : L,
}

impl<'a, D : Data, T : SomeTree<D>, L : Locator<D>> Slice<'a, D, T, L> where 
    for <'b> &'b mut T : SomeTreeRef<D>,
{
    pub fn new(tree : &'a mut T, locator : L) -> Self {
        Slice {
            phantom : PhantomData,
            tree,
            locator,
        }
    }

    pub fn summary(&mut self) -> D::Summary {
        self.tree.segment_summary(self.locator.clone())
    }

    pub fn act(&mut self, action : D::Action) {
        self.tree.act_segment(action, self.locator.clone());
    }

    pub fn search(self) -> <&'a mut T as SomeTreeRef<D>> :: Walker {
        methods::search(self.tree, self.locator)
    }
}

impl<'a, D : Data, T : SomeTree<D>, L : Locator<D>> Slice<'a, D, T, L> where 
    for<'b> &'b mut T : ModifiableTreeRef<D>,
{
    pub fn insert(&mut self, value : D::Value) -> Option<()> {
        let mut walker
         = methods::search(&mut *self.tree, self.locator.clone());
        walker.insert(value)
    }

    pub fn delete(&mut self) -> Option<D::Value> {
        let mut walker
         = methods::search(&mut *self.tree, self.locator.clone());
        walker.delete()
    }
}


impl<'a, D : Data, T : SomeTree<D>, L : Locator<D>> Slice<'a, D, T, L> where 
    for<'b> &'b mut T : SplittableTreeRef<D>,
{
    pub fn split_right(&mut self) -> Option<<<&mut T as SplittableTreeRef<D>>::SplittableWalker as SplittableWalker<D>>::T> {
        let mut walker
         = methods::search(&mut *self.tree, self.locator.clone());
        walker.split_right()
    }

    pub fn split_left(&mut self) -> Option<<<&mut T as SplittableTreeRef<D>>::SplittableWalker as SplittableWalker<D>>::T> {
        let mut walker
         = methods::search(&mut *self.tree, self.locator.clone());
        walker.split_left()
    }
}

