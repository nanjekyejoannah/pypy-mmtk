use crate::PyPy;
use mmtk::util::opaque_pointer::*;
use mmtk::vm::Collection;
use mmtk::vm::GCThreadContext;
use mmtk::Mutator;
use mmtk::MutatorContext;

pub struct VMCollection {}

impl Collection<PyPy> for VMCollection {
    fn stop_all_mutators<F>(_tls: VMWorkerThread, _mutator_visitor: F)
    where
        F: FnMut(&'static mut Mutator<PyPy>),
    {
        unimplemented!()
    }

    fn resume_mutators(_tls: VMWorkerThread) {
        unimplemented!()
    }

    fn block_for_gc(_tls: VMMutatorThread) {
        panic!("block_for_gc is not implemented")
    }

    fn spawn_gc_thread(_tls: VMThread, _ctx: GCThreadContext<PyPy>) {}

    fn prepare_mutator<T: MutatorContext<PyPy>>(
        _tls_w: VMWorkerThread,
        _tls_m: VMMutatorThread,
        _mutator: &T,
    ) {
        unimplemented!()
    }
}
