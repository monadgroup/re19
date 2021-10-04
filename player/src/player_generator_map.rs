use alloc::vec::Vec;
use core::mem;
use engine::animation::clip::{ClipReference, GeneratorClipMap};
use engine::generator::Generator;

pub struct PlayerGeneratorMap<'gen> {
    generators: Vec<Option<&'gen mut Generator>>,
}

impl<'gen> PlayerGeneratorMap<'gen> {
    pub fn new(generators: Vec<Option<&'gen mut Generator>>) -> Self {
        PlayerGeneratorMap { generators }
    }

    pub fn take<F: FnOnce(&mut Generator, &mut PlayerGeneratorMap)>(
        &mut self,
        index: usize,
        func: F,
    ) {
        let mut generator = None;
        mem::swap(&mut generator, &mut self.generators[index]);

        if let Some(unwrapped_gen) = generator {
            func(unwrapped_gen, self);
            mem::swap(&mut Some(unwrapped_gen), &mut self.generators[index]);
        }
    }
}

impl<'gen> GeneratorClipMap for PlayerGeneratorMap<'gen> {
    fn try_get_clip(&self, reference: ClipReference) -> Option<&dyn Generator> {
        let r = &self.generators[reference.clip_id() as usize];

        match r {
            Some(gen) => Some(*gen),
            None => None,
        }
    }

    fn try_get_clip_mut(&mut self, reference: ClipReference) -> Option<&mut dyn Generator> {
        let r = &mut self.generators[reference.clip_id() as usize];

        match r {
            Some(gen) => Some(*gen),
            None => None,
        }
    }
}
